// Copyright 2017 ThetaSinner
//
// This file is part of Osmium.
//
// Osmium is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Osmium is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
// 
// You should have received a copy of the GNU General Public License
// along with Osmium. If not, see <http://www.gnu.org/licenses/>.

// osmium
use http2::frame as framing;
use http2::stream::StreamId;

pub struct ConnectionFrameStateValidator {
    name: ConnectionFrameStateName
}

impl ConnectionFrameStateValidator {
    pub fn new() -> Self {
        ConnectionFrameStateValidator {
            name: ConnectionFrameStateName::AllowAny(ConnectionFrameState::new())
        }
    }

    // (6.2) A HEADERS frame without the END_HEADERS flag set MUST be followed by 
    // a CONTINUATION frame for the same stream. A receiver MUST treat the 
    // receipt of any other type of frame or a frame on a different stream 
    // as a connection error (Section 5.4.1) of type PROTOCOL_ERROR.
    pub fn is_okay(&mut self, frame_type: Option<framing::FrameType>, flags: u8, stream_id: StreamId) -> bool {
        let (new_name, okay) = match frame_type {
            Some(frame_type) => {
                match self.name {
                    ConnectionFrameStateName::AllowAny(ref s) => {
                        if frame_type == framing::FrameType::Headers && !framing::headers::is_end_headers(flags) {
                            (
                                Some(ConnectionFrameStateName::ReceiveContinuation(
                                    (s, stream_id).into()
                                )),
                                true
                            )
                        }
                        else {
                            (None, true)
                        }
                    },
                    ConnectionFrameStateName::ReceiveContinuation(ref s) => {
                        if stream_id == s.get_stream_id() && frame_type == framing::FrameType::Continuation {
                            if framing::continuation::is_end_headers(flags) {
                                (
                                    Some(ConnectionFrameStateName::AllowAny(s.into())),
                                    true
                                )
                            }
                            else {
                                (None, true)
                            }
                        }
                        else {
                            (None, false)
                        }
                    }
                }
            },
            None => {
                match self.name {
                    ConnectionFrameStateName::AllowAny(_) => {
                        // When allowing any frame, an unknown frame must be allowed and does not change the state.
                        (None, true)
                    },
                    ConnectionFrameStateName::ReceiveContinuation(_) => {
                        // When expecting a continuation frame, an unknown frame is a violation.
                        (None, false)
                    }
                }
            }
        };

        if let Some(new_name) = new_name {
            self.name = new_name;
        }

        okay
    }
}

pub enum ConnectionFrameStateName {
    AllowAny(ConnectionFrameState<StateAllowAny>),
    ReceiveContinuation(ConnectionFrameState<StateReceiveContinuation>)
}

pub struct ConnectionFrameState<S> {
    state: S
}

impl ConnectionFrameState<StateAllowAny> {
    pub fn new() -> Self {
        ConnectionFrameState {
            state: StateAllowAny
        }
    }
}

impl ConnectionFrameState<StateReceiveContinuation> {
    pub fn get_stream_id(&self) -> StreamId {
        self.state.stream_id
    }
}

pub struct StateAllowAny;

pub struct StateReceiveContinuation {
    stream_id: StreamId
}

impl<'a> From<(&'a ConnectionFrameState<StateAllowAny>, StreamId)> for ConnectionFrameState<StateReceiveContinuation> {
    fn from(val: (&ConnectionFrameState<StateAllowAny>, StreamId)) -> ConnectionFrameState<StateReceiveContinuation> {
        ConnectionFrameState {
            state: StateReceiveContinuation {
                stream_id: val.1
            }
        }
    }
}

impl<'a> From<&'a ConnectionFrameState<StateReceiveContinuation>> for ConnectionFrameState<StateAllowAny> {
    fn from(_: &ConnectionFrameState<StateReceiveContinuation>) -> ConnectionFrameState<StateAllowAny> {
        ConnectionFrameState {
            state: StateAllowAny {}
        }
    }
}
