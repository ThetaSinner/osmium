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

mod connection_frame_state;

// std
use std::collections::VecDeque;

// osmium
use http2::frame as framing;
use http2::error;

pub struct Connection {
    send_frames: VecDeque<Vec<u8>>,
    frame_state_validator: connection_frame_state::ConnectionFrameStateValidator
}

impl Connection {
    pub fn new() -> Connection {
        Connection {
            send_frames: VecDeque::new(),
            frame_state_validator: connection_frame_state::ConnectionFrameStateValidator::new()
        }
    }

    pub fn push_frame(&mut self, frame: framing::Frame) {
        // TODO handle frame type not recognised.
        let frame_type = match frame.header.frame_type {
            Some(ref frame_type) => frame_type.clone(),
            None => {
                // TODO this can be handled gracefully, no need to crash.
                panic!("cannot handle frame type not recognised");
            }
        };

        // Check that the incoming frame is what was expected on this connection.
        if !self.frame_state_validator.is_okay(frame_type.clone(), frame.header.flags, frame.header.stream_id) {
            // (6.2) A receiver MUST treat the receipt of any other type of frame 
            // or a frame on a different stream as a connection error (Section 5.4.1) 
            // of type PROTOCOL_ERROR.
            self.push_send_go_away_frame(error::HttpError::ConnectionError(
                error::ErrorCode::ProtocolError,
                error::ErrorName::HeaderBlockInterupted
            ));
            return;
        };

        match frame_type {
            framing::FrameType::Ping => {
                // (6.7) A PING frame with a stream identifier other than 0x0 is a connection error of type PROTOCOL_ERROR
                if frame.header.stream_id != 0x0 {
                    self.push_send_go_away_frame(error::HttpError::ConnectionError(
                        error::ErrorCode::ProtocolError,
                        error::ErrorName::StreamIdentifierOnConnectionFrame
                    ));
                    // (6.7) If a PING frame with a stream identifier other than 0x0 is received, then the recipient must
                    // respond with a connection error.
                    // That is, do not continue processing the ping.
                    return;
                }

                // decode the incoming ping frame
                let ping_frame_result = framing::ping::PingFrame::new(&frame.header, &mut frame.payload.into_iter());

                match ping_frame_result {
                    Ok(ping_frame) => {
                        if framing::ping::is_acknowledge(frame.header.flags) {
                            // TODO the server has no way of managing the connection thread. That is, the thread is only
                            // active when frames are received which means the connection is active and there's no point
                            // sending a ping.
                            panic!("can't handle ping response");
                        }
                        else {
                            // TODO add a second constructor method which builds a response.
                            let mut ping_response = framing::ping::PingFrameCompressModel::new();
                            ping_response.set_acknowledge();
                            ping_response.set_ping_payload(ping_frame.get_payload());

                            // (6.7) A PING frame with a stream identifier other than 0x0 is a connection error of type PROTOCOL_ERROR
                            self.push_send_frame(ping_response, 0x0);
                        }
                    },
                    Err(e) => {
                        // (6.7) the only error which decoding can produce is a FRAME_SIZE_ERROR, which is a connection error
                        // so it is correct to build a GO_AWAY frame from it.
                        self.push_send_go_away_frame(e);
                    }
                }
            },
            framing::FrameType::Headers => {
                // (6.2) A HEADERS frame which is not associated with a stream is a connection error of type PROTOCOL_ERROR
                if frame.header.stream_id == 0x0 {
                    self.push_send_go_away_frame(error::HttpError::ConnectionError(
                        error::ErrorCode::ProtocolError,
                        error::ErrorName::MissingStreamIdentifierOnStreamFrame
                    ));
                    return;
                }

                // TODO finish this
            }
            _ => {
                panic!("can't handle that frame type yet");
            }
        }
    }

    // Queues a frame to be sent.
    fn push_send_frame<T>(&mut self, frame: T, stream_id: framing::StreamId) where T : framing::CompressibleHttpFrame {
        self.send_frames.push_back(
            framing::compress_frame(frame, stream_id)
        );
    }

    fn push_send_go_away_frame(&mut self, http_error: error::HttpError) {
        // TODO send last stream processed. Steams are not implemented yet so this will have to wait. For now send
        // 0x0, which means no streams processed.
        let go_away = framing::go_away::GoAwayFrameCompressModel::new(0x0, http_error);

        // (6.8) A GOAWAY frame with a stream identifier other than 0x0 is a connection error of type PROTOCOL_ERROR.
        self.push_send_frame(go_away, 0x0);
    }

    pub fn pull_frame(&mut self) -> Option<Vec<u8>> {
        self.send_frames.pop_front()
    }
}
