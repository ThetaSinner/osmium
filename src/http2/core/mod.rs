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

// std
use std::collections::VecDeque;

// osmium
use http2::frame as framing;

pub struct Connection {
    send_frames: VecDeque<Vec<u8>>
}

impl Connection {
    pub fn new() -> Connection {
        Connection {
            send_frames: VecDeque::new()
        }
    }

    pub fn push_frame(&mut self, frame: framing::Frame) {
        // TODO handle frame type not recognised.
        let frame_type = match frame.header.frame_type {
            Some(ref frame_type) => frame_type.clone(),
            None => panic!("cannot handle frame type not recognised")
        };

        println!("{:?}", frame_type);

        match frame_type {
            &framing::FrameType::Ping => {
                // TODO check that stream id is 0, otherwise protocol error.

                let ping = framing::ping::PingFrame::new(&frame.header, &mut frame.payload.into_iter());

                if framing::ping::is_acknowledge(frame.header.flags) {
                    panic!("can't handle ping response");
                }
                else {
                    // TODO add a second constructor method which builds a response.
                    let mut ping_response = framing::ping::PingFrameCompressModel::new();
                    ping_response.set_acknowledge();
                    ping_response.set_ping_payload(ping.get_payload());

                    self.push_send_frame(ping_response, 0x0);
                }
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

    pub fn pull_frame(&mut self) -> Option<Vec<u8>> {
        self.send_frames.pop_front()
    }
}
