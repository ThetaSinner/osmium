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
pub mod stream_request;
pub mod stream_response;

pub use self::stream_request::StreamRequest;
pub use self::stream_response::StreamResponse;

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
use http2::hpack::{context as hpack_context, pack as hpack_pack};
use shared::server_trait;
use http2::core::ConnectionHandle;
use http2::core::ConnectionData;

// TODO break this file up!

// TODO can/should any of this data be moved into the state machine?

// TODO it is possible to send multiple data frames, it is necessary to make use of this to avoid
// sending payloads larger than the allowed size.

// TODO check that peer initiated streams use odd number identifiers.

// TODO while push promised must reference a peer initiated stream when created, nothing prevents many promises
// being generated on the same peer initiated stream. Therefore, it is necessary to handle the server running out of 
// stream identifiers to use. The client would just close the connection and open a new one, the server can do the
// same if it chooses. While waiting to kill the connection, should push promise be disabled?

pub struct Stream {
    id: framing::StreamId,

    state_name: state::StreamStateName,

    temp_header_block: Vec<u8>,

    request: StreamRequest,
    started_processing_request: bool,

    send_frames: Vec<Box<framing::CompressibleHttpFrame>>,

    connection_data: Rc<RefCell<ConnectionData>>,

    push_promise_queue: VecDeque<StreamRequest>,

    // Because these requests are being generated locally, the remote encoder will never encode them.
    // Therefore, it is necessary to keep them for use later without decoding.
    push_promise_publish_queue: VecDeque<(u32, StreamRequest)>,

    send_window: u32
    // TODO receive
}

impl Stream {
    pub fn new(id: framing::StreamId, connection_data: Rc<RefCell<ConnectionData>>) -> Self {
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

    pub fn new_promise(id: framing::StreamId, connection_data: Rc<RefCell<ConnectionData>>, request: StreamRequest) -> Self {
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
                            data_frame.get_payload().to_vec()
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
        // This is normally set when the request has been fully received. In this situation, set it as soon as the 
        // synthetic request has started to execute.
        self.started_processing_request = true;

        // Fetch the request from the state machine.
        let new_request = match self.state_name {
            state::StreamStateName::ReservedLocal(ref mut state) => {
                let mut new_request = StreamRequest::new();
                mem::swap(&mut state.state.stream_request, &mut new_request);

                new_request
            },
            _ => {
                panic!("state not handled");
            }
        };

        let response: StreamResponse = app.process(new_request.into(), Box::new(self)).into();

        // Notice that we do not handle push promise here. That is because promises must be initiated on a peer initiated stream,
        // which this stream will not be.

        self.send(response.to_frames(hpack_send_context));

        // TODO handle errors
        None
    }

    fn send(&mut self, frames: Vec<Box<framing::CompressibleHttpFrame>>) {
        let mut temp_send_frames = Vec::new();

        // TODO handle not sending if max frame size setting will be exceeded.

        let mut frame_iter = frames.into_iter();
        while let Some(frame) = frame_iter.next() {
            log_stream_send_frame!("Stream send", self.id, frame);

            let new_state = match self.state_name {
                state::StreamStateName::ReservedLocal(ref state) => {
                    match frame.get_frame_type() {
                        framing::FrameType::Headers => {
                            // (8.2.2) This stream becomes "half-closed" to the client (Section 5.1) after the initial HEADERS frame is sent.
                            let mut new_state = state::StreamStateName::HalfClosedRemote(state.into());

                            // Sending end stream is a seperate event, so if it is set, then we can have a second state transition here.
                            if framing::headers::is_end_stream(frame.get_flags()) {
                                new_state = match new_state {
                                    state::StreamStateName::HalfClosedRemote(ref state) => {
                                        state::StreamStateName::Closed(state.into())
                                    },
                                    _ => {
                                        unreachable!();
                                    }
                                };
                            }

                            temp_send_frames.push(frame);
                            
                            Some(new_state)
                        },
                        _ => {
                            // TODO what else can be sent here? it really should only be headers.
                            unimplemented!();
                        }
                    }
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

                            temp_send_frames.push(frame);
                            
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

                            temp_send_frames.push(frame);
                            
                            new_state
                        },
                        framing::FrameType::ResetStream => {
                            temp_send_frames.push(frame);

                            Some(state::StreamStateName::Closed(state.into()))
                        },
                        framing::FrameType::PushPromise => {
                            temp_send_frames.push(frame);

                            None
                        },
                        framing::FrameType::WindowUpdate => {
                            temp_send_frames.push(frame);

                            None
                        },
                        framing::FrameType::Continuation => {
                            // It would be nice to check that this continuation is a valid frame to be sending. But it's
                            // just not worth doing - the server must construct sequences of frames correctly.
                            temp_send_frames.push(frame);

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

                            temp_send_frames.push(frame);
                            
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

                            temp_send_frames.push(frame);
                            
                            new_state
                        },
                        framing::FrameType::ResetStream => {
                            temp_send_frames.push(frame);

                            Some(state::StreamStateName::Closed(state.into()))
                        },
                        framing::FrameType::PushPromise => {
                            temp_send_frames.push(frame);

                            None
                        },
                        framing::FrameType::WindowUpdate => {
                            temp_send_frames.push(frame);

                            None
                        },
                        framing::FrameType::Continuation => {
                            // It would be nice to check that this continuation is a valid frame to be sending. But it's
                            // just not worth doing - the server must construct sequences of frames correctly.
                            temp_send_frames.push(frame);

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

    pub fn fetch_send_frames(&mut self) -> Vec<Box<framing::CompressibleHttpFrame>> {
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
                let response: StreamResponse = app.process(new_request.into(), Box::new(self)).into();
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

impl ConnectionHandle for Stream {
    fn is_push_enabled(&self) -> bool {
        // TODO modify the stream to understand that it is a synthetic stream, and do not allow promises to be sent in that case.
        // TODO test that updating server push setting while running actually updates this value.
        self.connection_data.borrow().incoming_settings.enable_push
    }

    fn push_promise(&mut self, request: StreamRequest) -> Option<PushError> {
        self.queue_push_promise(request)
    }
}
