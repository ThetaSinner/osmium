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

pub struct Stream {
    state_name: state::StreamStateName,

    header_block: Vec<u8>,
    header_block_ready_to_process: bool,

    payload: String,
    payload_ready_to_process: bool,

    trailer_header_block: Vec<u8>,
    trailer_header_block_ready_to_process: bool,

    send_window: u32
}

impl Stream {
    pub fn new() -> Self {
        Stream {
            state_name: state::StreamStateName::Idle(state::StreamState::<state::StateIdle>::new()),

            header_block: Vec::new(),
            header_block_ready_to_process: false,

            payload: String::new(),
            payload_ready_to_process: false,

            trailer_header_block: Vec::new(),
            trailer_header_block_ready_to_process: false,

            send_window: 0
        }
    }

    pub fn recv(&mut self, frame: framing::StreamFrame) -> Option<error::HttpError> {
        let (opt_new_state, opt_err) = match self.state_name {
            state::StreamStateName::Idle(ref state) => {
                // (5.1) In the idle state we can receive headers and push promise frames.
                match frame.header.frame_type {
                    framing::FrameType::Headers => {
                        // The headers frame moves the stream into the open state.
                        let mut new_state = state::StreamStateName::Open(state.into());

                        // Decode and receive the header block.
                        let headers_frame = framing::headers::HeaderFrame::new(&frame.header, &mut frame.payload.into_iter());
                        self.header_block.extend(headers_frame.get_header_block_fragment());

                        if headers_frame.is_end_headers() {
                            self.header_block_ready_to_process = true;
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

                        self.payload = data_frame.get_payload().to_owned();
                        self.payload_ready_to_process = true;

                        (new_state, None)
                    },
                    framing::FrameType::Headers => {
                        // Decode and receive the header block.
                        let headers_frame = framing::headers::HeaderFrame::new(&frame.header, &mut frame.payload.into_iter());
                        self.trailer_header_block.extend(headers_frame.get_header_block_fragment());

                        if headers_frame.is_end_headers() {
                            self.trailer_header_block_ready_to_process = true;
                        }

                        if headers_frame.is_end_stream() {
                            (
                                Some(
                                    state::StreamStateName::HalfClosedRemote(state.into())
                                ),
                                None
                            )
                        }
                        else {
                            // (8.1) An endpoint that receives a HEADERS frame without the END_STREAM 
                            // flag set after receiving a final (non-informational) status code MUST 
                            // treat the corresponding request or response as malformed (Section 8.1.2.6).
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
                        // verifies that headers are followed by continuation frames.
                        // TODO However, still need to check for continuation frames which are floating about
                        // on their own. Note that this condition might actually catch this case.

                        if self.header_block_ready_to_process {
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
                            self.header_block.extend(continuation_frame.get_header_block_fragment());

                            if continuation_frame.is_end_headers() {
                                self.header_block_ready_to_process = true;
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
            _ => {
                panic!("state not handled yet");
                (None, None)
            }
        };

        if self.header_block_ready_to_process {
            self.process_header_block();
        }

        if let Some(new_state) = opt_new_state {
            self.state_name = new_state;
        }

        opt_err
    }

    pub fn send(&mut self) {

    }

    fn process_header_block(&mut self) {
        // TODO use the hpack module to process the header block
    }
}
