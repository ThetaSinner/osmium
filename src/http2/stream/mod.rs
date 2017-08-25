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
    header_block_read_to_process: bool
}

impl Stream {
    pub fn new() -> Self {
        Stream {
            state_name: state::StreamStateName::Idle(state::StreamState::<state::StateIdle>::new()),
            header_block: Vec::new(),
            header_block_read_to_process: false
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
                            self.header_block_read_to_process = true;
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
                        panic!("can't handle push promise yet");
                        (None, None)
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
            _ => {
                panic!("state not handled yet");
                (None, None)
            }
        };

        if self.header_block_read_to_process {
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
