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

// std
use std::convert;
use std::mem;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::VecDeque;

// osmium
use http2::frame as framing;
use http2::error;
use http2::header;
use http2::hpack::{context as hpack_context, unpack as hpack_unpack, pack as hpack_pack};
use shared::server_trait;
use http2::core::ConnectionHandle;
use http2::core::ConnectionData;

// TODO break this file up!

#[derive(Debug)]
pub struct StreamRequest {
    pub headers: header::Headers,
    pub payload: Option<String>,
    pub trailer_headers: Option<header::Headers>
}

#[derive(Debug)]
pub struct StreamResponse {
    pub informational_headers: Vec<header::Headers>,
    pub headers: header::Headers,
    pub payload: Option<String>,
    pub trailer_headers: Option<header::Headers>
}

impl StreamRequest {
    pub fn new() -> Self {
        StreamRequest {
            headers: header::Headers::new(),
            payload: None,
            trailer_headers: None
        }
    }

    // TODO will return an error.
    pub fn process_temp_header_block(&mut self, temp_header_block: &[u8], hpack_recv_context: &mut hpack_context::RecvContext) {
        let decoded = hpack_unpack::unpack(temp_header_block, hpack_recv_context);

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

impl StreamResponse {
    pub fn to_frames(self, hpack_send_context: &mut hpack_context::SendContext) -> Vec<Box<framing::CompressibleHttpFrame>>
    {
        trace!("Starting to convert stream response to frames [{:?}]", self);

        let mut frames: Vec<Box<framing::CompressibleHttpFrame>> = Vec::new();

        for informational_header in &self.informational_headers {
            frames.extend(
                StreamResponse::headers_to_frames(informational_header, hpack_send_context, false)
            );
        }

        let headers = StreamResponse::headers_to_frames(&self.headers, hpack_send_context, self.payload.is_none() && self.trailer_headers.is_none());
        frames.extend(headers);

        if self.payload.is_some() {
            let mut data_frame = framing::data::DataFrameCompressModel::new(false);
            data_frame.set_payload(self.payload.unwrap().into_bytes());
            if self.trailer_headers.is_none() {
                data_frame.set_end_stream();
            }
            frames.push(Box::new(data_frame));
        }

        if self.trailer_headers.is_some() {
            let trailer_headers_frame = StreamResponse::headers_to_frames(&self.trailer_headers.unwrap(), hpack_send_context, true);
            frames.extend(trailer_headers_frame);
        }

        trace!("Converted to frames [{:?}]", frames);

        frames
    }

    fn headers_to_frames(headers: &header::Headers, hpack_send_context: &mut hpack_context::SendContext, end_stream: bool) -> Vec<Box<framing::CompressibleHttpFrame>>
    {
        let mut temp_frames: Vec<Box<framing::CompressibleHttpFrame>> = Vec::new();

        let packed = hpack_pack::pack(&headers, hpack_send_context, true);
        let num_chunks = ((packed.len() as f32) / 150f32).ceil() as i32;
        let mut chunk_count = 1;
        let mut chunks = packed.chunks(150);

        if let Some(first_chunk) = chunks.next() {
            let mut headers_frame = framing::headers::HeadersFrameCompressModel::new(false, false);
            if end_stream {
                headers_frame.set_end_stream();
            }
            if num_chunks == 1 {
                headers_frame.set_end_headers();
            }
            headers_frame.set_header_block_fragment(first_chunk.to_vec());
            temp_frames.push(Box::new(headers_frame));
        }
        else {
            panic!("empty headers block");
        }
        
        while let Some(chunk) = chunks.next() {
            let mut headers_frame = framing::headers::HeadersFrameCompressModel::new(false, false);
            headers_frame.set_header_block_fragment(chunk.to_vec());

            chunk_count += 1;
            if chunk_count == num_chunks {
                headers_frame.set_end_headers();
            }

            temp_frames.push(Box::new(headers_frame));
        }

        temp_frames
    }
}

pub struct Stream {
    id: u32,

    state_name: state::StreamStateName,

    temp_header_block: Vec<u8>,

    request: StreamRequest,
    started_processing_request: bool,

    send_frames: Vec<Vec<u8>>,

    connection_data: Rc<RefCell<ConnectionData>>,

    push_promise_queue: VecDeque<StreamRequest>,

    // Because these requests are being generated locally, the remote encoder will never encode them.
    // Therefore, it is necessary to keep them for use later without decoding.
    push_promise_publish_queue: VecDeque<(u32, StreamRequest)>,

    send_window: u32
}

impl Stream {
    pub fn new(id: u32, connection_data: Rc<RefCell<ConnectionData>>) -> Self {
        Stream {
            id: id, 

            state_name: state::StreamStateName::Idle(state::StreamState::<state::StateIdle>::new()),

            temp_header_block: Vec::new(),

            request: StreamRequest {
                headers: header::Headers::new(),
                payload: None,
                trailer_headers: None
            },
            started_processing_request: false,

            send_frames: Vec::new(),

            connection_data: connection_data,

            push_promise_queue: VecDeque::new(),
            push_promise_publish_queue: VecDeque::new(),

            send_window: 0
        }
    }

    pub fn new_promise(id: u32, connection_data: Rc<RefCell<ConnectionData>>, request: StreamRequest) -> Self {
        let mut promised_stream = Stream::new(id, connection_data);

        promised_stream.state_name = if let state::StreamStateName::Idle(ref state) = promised_stream.state_name {
            state::StreamStateName::ReservedLocal((state, request).into())
        }
        else {
            panic!("guess the dev should have fixed this");
        };

        promised_stream
    }

    // Note that unpacking headers is stateful, and we can only borrow the connection's context mutably once.
    pub fn recv<T, R, S>(
        &mut self, 
        frame: framing::StreamFrame,
        hpack_send_context: &mut hpack_context::SendContext,
        hpack_recv_context: &mut hpack_context::RecvContext,
        app: &T
    ) -> Option<error::HttpError>
        where T: server_trait::OsmiumServer<Request=R, Response=S>, 
              R: convert::From<StreamRequest>,
              S: convert::Into<StreamResponse>
    {
        log_stream_recv!("Receive frame", self.id, self.state_name, frame);

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
                            self.request.process_temp_header_block(self.temp_header_block.as_slice(), hpack_recv_context);

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
                            };
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
                            self.request.process_temp_header_block(self.temp_header_block.as_slice(), hpack_recv_context);

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
                        let window_update_frame = framing::window_update::WindowUpdateFrame::new_stream(&frame.header, &mut frame.payload.into_iter());

                        // TODO handle window frame decode error.

                        if window_update_frame.get_window_size_increment() == 0 {
                            // (6.9) A receiver MUST treat the receipt of a WINDOW_UPDATE frame with an flow-control 
                            // window increment of 0 as a stream error (Section 5.4.2) of type PROTOCOL_ERROR
                            (
                                None,
                                Some(
                                    error::HttpError::StreamError(
                                        error::ErrorCode::ProtocolError,
                                        error::ErrorName::ZeroWindowSizeIncrement
                                    )
                                )
                            )
                        }
                        else {
                            self.send_window += window_update_frame.get_window_size_increment();
                            (None, None)
                        }
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
                                self.request.process_temp_header_block(self.temp_header_block.as_slice(), hpack_recv_context);
                                
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
                                self.request.process_temp_header_block(self.temp_header_block.as_slice(), hpack_recv_context);
                                
                                // TODO this only removes values from the vector, it doesn't change the allocated capacity.
                                self.temp_header_block.clear();
                            }

                            // TODO handle continuation frame decode error.
                            (None, None)
                        }
                    },
                    framing::FrameType::WindowUpdate => {
                        let window_update_frame = framing::window_update::WindowUpdateFrame::new_stream(&frame.header, &mut frame.payload.into_iter());

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
            state::StreamStateName::Closed(_) => {
                match frame.header.frame_type {
                    framing::FrameType::Priority => {
                        // TODO log and discard instead panic.
                        unimplemented!();
                    },
                    framing::FrameType::WindowUpdate => {
                        // (5.1) Endpoints MUST ignore WINDOW_UPDATE or RST_STREAM frames received in this state
                        (None, None)
                    },
                    framing::FrameType::ResetStream => {
                        // (5.1) Endpoints MUST ignore WINDOW_UPDATE or RST_STREAM frames received in this state
                        (None, None)
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
            }
        };

        println!("where's the stream at? {:?}", self.request);

        if let Some(new_state) = opt_new_state {
            self.state_name = new_state;
        }

        log_stream_post_recv!("Post receive", self.id, self.state_name);

        // TODO should not try to process if an error occurred?
        // Process the request if it is fully received.
        self.try_start_process(app, hpack_send_context);

        opt_err
    }

    pub fn recv_promised<T, R, S>(
        &mut self,
        hpack_send_context: &mut hpack_context::SendContext,
        app: &T
    ) -> Option<error::HttpError>
        where T: server_trait::OsmiumServer<Request=R, Response=S>, 
              R: convert::From<StreamRequest>,
              S: convert::Into<StreamResponse>
    {
        let (new_state, new_request) = match self.state_name {
            state::StreamStateName::ReservedLocal(ref mut state) => {
                self.started_processing_request = true;

                let mut new_request = StreamRequest::new();
                mem::swap(&mut state.state.stream_request, &mut new_request);

                (
                    state::StreamStateName::HalfClosedRemote(state.into()),
                    new_request
                )
            },
            _ => {
                panic!("state not handled");
            }
        };

        let response: StreamResponse = app.process(new_request.into(), Box::new(&self)).into();

        while let Some(request) = self.push_promise_queue.pop_back() {
            let mut push_promise_frame = framing::push_promise::PushPromiseFrameCompressModel::new(true);

            let promised_stream_identifier = self.connection_data.borrow_mut().get_next_server_created_stream_id();
            push_promise_frame.set_promised_stream_identifier(
                promised_stream_identifier
            );
            push_promise_frame.set_header_block_fragment(
                hpack_pack::pack(&request.headers, hpack_send_context, true)
            );

            self.push_promise_publish_queue.push_front((promised_stream_identifier, request));

            self.send(vec![Box::new(push_promise_frame)]);
        }

        self.send(response.to_frames(hpack_send_context));

        self.state_name = new_state;

        // TODO handle errors
        None
    }

    fn send(&mut self, frames: Vec<Box<framing::CompressibleHttpFrame>>) {
        let mut temp_send_frames = Vec::new();

        let mut frame_iter = frames.into_iter();
        while let Some(frame) = frame_iter.next() {
            let new_state = match self.state_name {
                state::StreamStateName::ReservedLocal(_) => {
                    // This is a state the server will handle, but it's not implemented yet.
                    unimplemented!();
                },
                state::StreamStateName::Open(ref state) => {
                    match frame.get_frame_type() {
                        framing::FrameType::Data => {
                            let new_state = if framing::data::is_end_stream(frame.get_flags()) {
                                Some(
                                    state::StreamStateName::HalfClosedLocal(state.into())
                                )
                            }
                            else {
                                None
                            };

                            temp_send_frames.push(
                                framing::compress_frame(frame, self.id)
                            );
                            
                            new_state
                        },
                        framing::FrameType::Headers => {
                            let new_state = if framing::headers::is_end_stream(frame.get_flags()) {
                                Some(
                                    state::StreamStateName::HalfClosedLocal(state.into())
                                )
                            }
                            else {
                                None
                            };

                            temp_send_frames.push(
                                framing::compress_frame(frame, self.id)
                            );
                            
                            new_state
                        },
                        framing::FrameType::ResetStream => {
                            temp_send_frames.push(
                                framing::compress_frame(frame, self.id)
                            );

                            Some(state::StreamStateName::Closed(state.into()))
                        },
                        framing::FrameType::PushPromise => {
                            temp_send_frames.push(
                                framing::compress_frame(frame, self.id)
                            );

                            None
                        },
                        framing::FrameType::WindowUpdate => {
                            temp_send_frames.push(
                                framing::compress_frame(frame, self.id)
                            );

                            None
                        },
                        framing::FrameType::Continuation => {
                            // The continuation frames are appended to form a block of frames. The header and continuation
                            // frames must be sent with no frames from this or other streams interleaved. The simplest way
                            // to achieve this is to clump them together. There seems to be no advantage to leaving them
                            // as seperate frames and trying to coordinate this later.
                            if let Some(ref mut last_frame) = temp_send_frames.last_mut() {
                                last_frame.extend(
                                    framing::compress_frame(frame, self.id)
                                );
                            }
                            else {
                                panic!("continuation with no preceeding frame to send");
                            }

                            None
                        },
                        _ => {
                            // TODO the frames which should be handled have been, this should be an internal error.
                            panic!("unhandled frame for send");
                        }
                    }
                },
                state::StreamStateName::HalfClosedRemote(ref state) => {
                    match frame.get_frame_type() {
                        framing::FrameType::Data => {
                            let new_state = if framing::data::is_end_stream(frame.get_flags()) {
                                Some(
                                    state::StreamStateName::Closed(state.into())
                                )
                            }
                            else {
                                None
                            };

                            temp_send_frames.push(
                                framing::compress_frame(frame, self.id)
                            );
                            
                            new_state
                        },
                        framing::FrameType::Headers => {
                            let new_state = if framing::headers::is_end_stream(frame.get_flags()) {
                                Some(
                                    state::StreamStateName::Closed(state.into())
                                )
                            }
                            else {
                                None
                            };

                            temp_send_frames.push(
                                framing::compress_frame(frame, self.id)
                            );
                            
                            new_state
                        },
                        framing::FrameType::ResetStream => {
                            temp_send_frames.push(
                                framing::compress_frame(frame, self.id)
                            );

                            Some(state::StreamStateName::Closed(state.into()))
                        },
                        framing::FrameType::PushPromise => {
                            temp_send_frames.push(
                                framing::compress_frame(frame, self.id)
                            );

                            None
                        },
                        framing::FrameType::WindowUpdate => {
                            temp_send_frames.push(
                                framing::compress_frame(frame, self.id)
                            );

                            None
                        },
                        framing::FrameType::Continuation => {
                            // The continuation frames are appended to form a block of frames. The header and continuation
                            // frames must be sent with no frames from this or other streams interleaved. The simplest way
                            // to achieve this is to clump them together. There seems to be no advantage to leaving them
                            // as seperate frames and trying to coordinate this later.
                            if let Some(ref mut last_frame) = temp_send_frames.last_mut() {
                                last_frame.extend(
                                    framing::compress_frame(frame, self.id)
                                );
                            }
                            else {
                                panic!("continuation with no preceeding frame to send");
                            }

                            None
                        },
                        _ => {
                            // TODO the frames which should be handled have been, this should be an internal error.
                            panic!("unhandled frame for send");
                        }
                    }
                },
                _ => {
                    // TODO there's more to handle here.
                    panic!("unhandled state for send");
                }
            };

            if let Some(new_state) = new_state {
                self.state_name = new_state;
            }
        }

        self.send_frames.extend(temp_send_frames);

        trace!("Finished sending frames on stream [{:?}]", self.send_frames);
    }

    pub fn fetch_push_promise(&mut self) -> Option<(u32, StreamRequest)> {
        self.push_promise_publish_queue.pop_back()
    }

    pub fn fetch_send_frames(&mut self) -> Vec<Vec<u8>> {
        self.send_frames.drain(0..).collect()
    }

    fn should_headers_frame_end_stream(&self) -> bool {
        // If the request headers have already been received, but another headers frame is
        // being processed then is must end the stream.
        !self.request.headers.is_empty()
    }

    fn try_start_process<T, R, S>(&mut self, app: &T, hpack_send_context: &mut hpack_context::SendContext) 
        where T: server_trait::OsmiumServer<Request=R, Response=S>,
              R: convert::From<StreamRequest>,
              S: convert::Into<StreamResponse>
    {
        match self.state_name {
            state::StreamStateName::HalfClosedRemote(_) => {
                if !self.temp_header_block.is_empty() {
                    // There is a header block being received so the request must not be processed
                    // yet.
                    return;
                }

                // As soon as we've started processing, this flag needs to have been set to true.
                // This allows the server to tell the client which streams have started to be processed
                // in the event of an error.
                self.started_processing_request = true;

                let mut new_request = StreamRequest::new();
                mem::swap(&mut self.request, &mut new_request);

                trace!("Passing request to the application [{:?}]", new_request);
                // TODO should the application be allowed to error?
                let response: StreamResponse = app.process(new_request.into(), Box::new(&self)).into();
                trace!("Got response from the application [{:?}]", response);

                // TODO this has been duplicated.
                while let Some(request) = self.push_promise_queue.pop_back() {
                    let mut push_promise_frame = framing::push_promise::PushPromiseFrameCompressModel::new(true);

                    let promised_stream_identifier = self.connection_data.borrow_mut().get_next_server_created_stream_id();
                    push_promise_frame.set_promised_stream_identifier(
                        promised_stream_identifier
                    );
                    push_promise_frame.set_header_block_fragment(
                        hpack_pack::pack(&request.headers, hpack_send_context, true)
                    );

                    self.push_promise_publish_queue.push_front((promised_stream_identifier, request));

                    self.send(vec![Box::new(push_promise_frame)]);
                }

                self.send(response.to_frames(hpack_send_context));
            },
            _ => {
                // Request not fully received, do nothing.
            }
        }
    }

    fn queue_push_promise(&mut self, request: StreamRequest) -> Option<PushError> {
        self.push_promise_queue.push_front(request);

        // TODO handle errors.
        None
    }
}

use http2::core::PushError;

impl<'a> ConnectionHandle for &'a mut Stream {
    fn is_push_enabled(&self) -> bool {
        // TODO test that updating server push setting while running actually updates this value.
        self.connection_data.borrow().incoming_settings.enable_push
    }

    fn push_promise(&mut self, request: StreamRequest) -> Option<PushError> {
        self.queue_push_promise(request)
    }
}
