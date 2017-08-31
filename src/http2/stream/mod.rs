// Copyright 2017 ThetaSinner
//
// This file is part of Osmium.

// Osmium is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Osmium is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Osmium. If not, see <http://www.gnu.org/licenses/>.

pub mod state;

use http2::frame as framing;
use http2::error;
use http2::header;
use http2::hpack::{context as hpack_context, unpack as hpack_unpack};

#[derive(Debug)]
pub struct StreamRequest {
    headers: header::Headers,
    payload: Option<String>,
    trailer_headers: Option<header::Headers>
}

impl StreamRequest {
    // TODO will return an error.
    pub fn process_temp_header_block(&mut self, temp_header_block: &[u8], hpack_context: &mut hpack_context::Context) {
        let decoded = hpack_unpack::unpack(temp_header_block, hpack_context);

        // TODO can the header block be empty? because that will break the logic below.

        if self.headers.is_empty() {
            // If no request headers have been received then these are the request headers.
            self.headers = decoded.headers;
        }
        else if self.trailer_headers.is_none() {
            // If no trailer headers have been received then these are the tailer headers.
            self.trailer_headers = Some(decoded.headers);
        }
        else {
            // TODO handle error. We have received all the header blocks we were expecting, but received
            // a request to process another.
            panic!("unexpected header block");
        }
    }
}

pub struct Stream {
    state_name: state::StreamStateName,

    temp_header_block: Vec<u8>,

    request: StreamRequest,
    started_processing_request: bool

    send_window: u32
}

impl Stream {
    pub fn new() -> Self {
        Stream {
            state_name: state::StreamStateName::Idle(state::StreamState::<state::StateIdle>::new()),

            temp_header_block: Vec::new(),

            request: StreamRequest {
                headers: header::Headers::new(),
                payload: None,
                trailer_headers: None
            },
            started_processing_request: false,

            send_window: 0
        }
    }

    // Note that unpacking headers is stateful, and we can only borrow the connection's context mutably once.
    pub fn recv(&mut self, frame: framing::StreamFrame, hpack_context: &mut hpack_context::Context) -> Option<error::HttpError> {
        // TODO used a named tuple for this so that the errors are better and 
        // it is clearer where the yields are in the block below.
        let (opt_new_state, opt_err) = match self.state_name {
            state::StreamStateName::Idle(ref state) => {
                // (5.1) In the idle state we can receive headers and push promise frames.
                match frame.header.frame_type {
                    framing::FrameType::Headers => {
                        // The headers frame moves the stream into the open state.
                        let mut new_state = state::StreamStateName::Open(state.into());

                        // Decode and receive the header block.
                        let headers_frame = framing::headers::HeaderFrame::new(&frame.header, &mut frame.payload.into_iter());
                        self.temp_header_block.extend(headers_frame.get_header_block_fragment());

                        // If the headers block is complete then unpack it immediately.
                        if headers_frame.is_end_headers() {
                            self.request.process_temp_header_block(self.temp_header_block.as_slice(), hpack_context);

                            // TODO this only removes values from the vector, it doesn't change the allocated capacity.
                            self.temp_header_block.clear();
                        }

                        if headers_frame.is_end_stream() {
                            // This is an interesting consequence of using enums for wrapping states.
                            // This can never fail, because we have explicitly changed state to open above.
                            // But we still have to destructure.
                            new_state = if let state::StreamStateName::Open(ref state) = new_state {
                                 state::StreamStateName::HalfClosedRemote(state.into())
                            }
                            else {
                                unreachable!("enum decomposition failed. How?");
                            }
                        }

                        (Some(new_state), None)
                    },
                    framing::FrameType::PushPromise => {
                        // (8.2) A client cannot push. Thus, servers MUST treat the receipt of a 
                        // PUSH_PROMISE frame as a connection error (Section 5.4.1) of type PROTOCOL_ERROR.
                        (
                            None,
                            Some(
                                error::HttpError::ConnectionError(
                                    error::ErrorCode::ProtocolError,
                                    error::ErrorName::CannotPushToServer
                                )
                            )
                        )
                    },
                    _ => {
                        // (5.1) Receiving any other frame than headers or push promise in this state
                        // must be treated as a connection error (Section 5.4.1) of type PROTOCOL_ERROR.
                        (
                            None,
                            Some(
                                error::HttpError::ConnectionError(
                                    error::ErrorCode::ProtocolError,
                                    error::ErrorName::StreamStateVoilation
                                )
                            )
                        )
                    }
                }
            },
            state::StreamStateName::Open(ref state) => {
                match frame.header.frame_type {
                    framing::FrameType::Data => {
                        let data_frame = framing::data::DataFrame::new(&frame.header, &mut frame.payload.into_iter());

                        // If the client ended the stream then it becomes half closed remote.
                        let new_state = if data_frame.is_end_stream() {
                            Some(
                                state::StreamStateName::HalfClosedRemote(state.into())
                            )
                        }
                        else {
                            None
                        };

                        self.request.payload = Some(
                            data_frame.get_payload().to_owned()
                        );

                        (new_state, None)
                    },
                    framing::FrameType::Headers => {
                        // Decode and receive the header block.
                        let headers_frame = framing::headers::HeaderFrame::new(&frame.header, &mut frame.payload.into_iter());
                        self.temp_header_block.extend(headers_frame.get_header_block_fragment());

                        // This needs to be checked before possibly processing this headers. Processing the headers
                        // changes the state and therefore MAY change this value. Checking before is correct, because
                        // it is this header frame which needs to be checked for end stream rather than the next one.
                        let should_end_stream = self.should_headers_frame_end_stream();

                        if headers_frame.is_end_headers() {
                            self.request.process_temp_header_block(self.temp_header_block.as_slice(), hpack_context);

                            // TODO this only removes values from the vector, it doesn't change the allocated capacity.
                            self.temp_header_block.clear();
                        }

                        // (8.1) An endpoint that receives a HEADERS frame without the END_STREAM 
                        // flag set after receiving a final (non-informational) status code MUST 
                        // treat the corresponding request or response as malformed (Section 8.1.2.6).
                        if should_end_stream {
                            if headers_frame.is_end_stream() {
                                (
                                    Some(
                                        state::StreamStateName::HalfClosedRemote(state.into())
                                    ),
                                    None
                                )
                            }
                            else {
                                (
                                    None,
                                    Some(
                                        error::HttpError::StreamError(
                                            error::ErrorCode::ProtocolError,
                                            error::ErrorName::TrailerHeaderBlockShouldTerminateStream
                                        )
                                    )
                                )
                            }
                        }
                        else {
                            (None, None)
                        }
                    },
                    framing::FrameType::Priority => {
                        unimplemented!();
                    },
                    framing::FrameType::ResetStream => {
                        let reset_stream_frame = framing::reset_stream::ResetStreamFrame::new(&frame.header, &mut frame.payload.into_iter());

                        // Log the error code for the stream reset.
                        if let Some(error_code) = error::to_error_code(reset_stream_frame.get_error_code()) {
                            warn!("Stream reset, error code {:?}", error_code);
                        }
                        else {
                            error!("Stream was reset, with unrecognised error code");
                        }

                        (
                            Some(state::StreamStateName::Closed(state.into())),
                            None
                        )
                    },
                    framing::FrameType::WindowUpdate => {
                        // (6.9) The WINDOW_UPDATE frame can be specific to a stream or to the 
                        // entire connection. In the former case, the frame's stream identifier 
                        // indicates the affected stream; in the latter, the value "0" 
                        // indicates that the entire connection is the subject of the frame.
                        
                        // Given the above we just assume that this window update is for this stream,
                        // otherwise the frame wouldn't have been send to this stream.
                        let window_update_frame = framing::window_update::WindowUpdateFrame::new(&frame.header, &mut frame.payload.into_iter());

                        self.send_window += window_update_frame.get_window_size_increment();

                        // TODO there is an error to be handled here if the frame decode fails.
                        (None, None)
                    },
                    framing::FrameType::Continuation => {
                        // TODO the server doesn't expect an arbitrary number of informational header blocks,
                        // these only appear on responses.

                        // The continuation frame must be preceded by a headers or push promise, without the
                        // end headers flag set. The server will not accept push promise and the connection
                        // verifies that headers are followed by continuation frames. This condition checks
                        // that this continuation is preceded by another frame which contained headers.
                        if self.temp_header_block.is_empty() {
                            // (6.10) A CONTINUATION frame MUST be preceded by a HEADERS, PUSH_PROMISE or 
                            // CONTINUATION frame without the END_HEADERS flag set. 
                            // A recipient that observes violation of this rule MUST respond with a 
                            // connection error (Section 5.4.1) of type PROTOCOL_ERROR
                            (
                                None,
                                Some(
                                    error::HttpError::ConnectionError(
                                        error::ErrorCode::ProtocolError,
                                        error::ErrorName::UnexpectedContinuationFrame
                                    )
                                )
                            )
                        }
                        else {
                            let continuation_frame = framing::continuation::ContinuationFrame::new(&frame.header, &mut frame.payload.into_iter());
                            self.temp_header_block.extend(continuation_frame.get_header_block_fragment());

                            if continuation_frame.is_end_headers() {
                                self.request.process_temp_header_block(self.temp_header_block.as_slice(), hpack_context);
                                
                                // TODO this only removes values from the vector, it doesn't change the allocated capacity.
                                self.temp_header_block.clear();
                            }

                            // TODO handle continuation frame decode error.
                            (None, None)
                        }
                    },
                    _ => {
                        // Any frame is valid in this state, but we must handle all type frame types in the enum to make rust happy.
                        // TODO this should be an internal server error if one of the types not covered is actually received here.
                        (
                            None,
                            Some(
                                error::HttpError::ConnectionError(
                                    error::ErrorCode::ProtocolError,
                                    error::ErrorName::StreamStateVoilation
                                )
                            )
                        )
                    }
                }
            },
            state::StreamStateName::HalfClosedRemote(ref state) => {
                match frame.header.frame_type {
                    // This looks like a deviation from the spec. In fact it's not. Continuation frames are 
                    // logically part of another frame. However, the first frame in the sequence may half close
                    // the stream. This means that the state will transition before the headers are fully
                    // received. So the continuation frames are received here if and only if a headers has
                    // already started to be received.
                    // This is invisible to the peer.
                    framing::FrameType::Continuation => {
                        if self.temp_header_block.is_empty() {
                            // TODO This should possibly return a stream error of type stream closed, because this 
                            // frame should not have been sent in this state.

                            // (6.10) A CONTINUATION frame MUST be preceded by a HEADERS, PUSH_PROMISE or 
                            // CONTINUATION frame without the END_HEADERS flag set. 
                            // A recipient that observes violation of this rule MUST respond with a 
                            // connection error (Section 5.4.1) of type PROTOCOL_ERROR
                            (
                                None,
                                Some(
                                    error::HttpError::ConnectionError(
                                        error::ErrorCode::ProtocolError,
                                        error::ErrorName::UnexpectedContinuationFrame
                                    )
                                )
                            )
                        }
                        else {
                            let continuation_frame = framing::continuation::ContinuationFrame::new(&frame.header, &mut frame.payload.into_iter());
                            self.temp_header_block.extend(continuation_frame.get_header_block_fragment());

                            if continuation_frame.is_end_headers() {
                                self.request.process_temp_header_block(self.temp_header_block.as_slice(), hpack_context);
                                
                                // TODO this only removes values from the vector, it doesn't change the allocated capacity.
                                self.temp_header_block.clear();
                            }

                            // TODO handle continuation frame decode error.
                            (None, None)
                        }
                    },
                    framing::FrameType::WindowUpdate => {
                        let window_update_frame = framing::window_update::WindowUpdateFrame::new(&frame.header, &mut frame.payload.into_iter());

                        self.send_window += window_update_frame.get_window_size_increment();

                        // TODO there is an error to be handled here if the frame decode fails.
                        (None, None)
                    },
                    framing::FrameType::Priority => {
                        unimplemented!();
                    },
                    framing::FrameType::ResetStream => {
                        let reset_stream_frame = framing::reset_stream::ResetStreamFrame::new(&frame.header, &mut frame.payload.into_iter());

                        // Log the error code for the stream reset.
                        if let Some(error_code) = error::to_error_code(reset_stream_frame.get_error_code()) {
                            warn!("Stream reset, error code {:?}", error_code);
                        }
                        else {
                            error!("Stream was reset, with unrecognised error code");
                        }

                        (
                            Some(state::StreamStateName::Closed(state.into())),
                            None
                        )
                    },
                    _ => {
                        // (5.1) If an endpoint receives additional frames, other than WINDOW_UPDATE, PRIORITY, 
                        // or RST_STREAM, for a stream that is in this state, it MUST respond with a stream 
                        // error (Section 5.4.2) of type STREAM_CLOSED.
                        (
                            None,
                            Some(
                                error::HttpError::StreamError(
                                    error::ErrorCode::StreamClosed,
                                    error::ErrorName::UnexpectedFrameOnHalfClosedStream
                                )
                            )
                        )
                    }
                }
            },
            state::StreamStateName::Closed(ref state) => {
                match frame.header.frame_type {
                    framing::FrameType::Priority => {
                        // TODO log and discard instead panic.
                        unimplemented!();
                    },
                    _ => {
                        // TODO there is more to do here. There is a small race condition, where the stream might be closed
                        // because we've sent a reset or end stream but they haven't been received by the peer.
                        // There are some rules about what can be discarded without error, and what is a more serious error.

                        // (5.1) An endpoint that receives any frame other than PRIORITY after receiving 
                        // a RST_STREAM MUST treat that as a stream error (Section 5.4.2) of type STREAM_CLOSED.
                        (
                            None,
                            Some(
                                error::HttpError::StreamError(
                                    error::ErrorCode::StreamClosed,
                                    error::ErrorName::StreamIsClosed
                                )
                            )
                        )
                    }
                }
            },
            _ => {
                // The following states are not handled.

                // Reserved remote: Cannot enter this state, because any incoming push promise to the 
                // server will be rejected.

                // Half closed local: No access from reserved remote, as above. Therefore, this state can only be 
                // reached if we send end stream before the client ends stream.
                // TODO This might happen if the request is processed before the trailer headers are received for
                // example. So this needs to go down as a case to be handled later.

                panic!("state not handled yet");
                (None, None)
            }
        };

        println!("where's the stream at? {:?}", self.request);

        if let Some(new_state) = opt_new_state {
            self.state_name = new_state;
        }

        opt_err
    }

    pub fn send(&mut self) {

    }

    fn should_headers_frame_end_stream(&self) -> bool {
        // If the request headers have already been received, but another headers frame is
        // being processed then is must end the stream.
        !self.request.headers.is_empty()
    }
}
