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
use std::vec::IntoIter;

// osmium
use super::CompressibleHttpFrame;
use super::FrameType;
use http2::error;

const FLAG_ACK: u8 = 0x1;

// the spec says '8 octets of opaque data', which can be anything? or should it be random
const DEFAULT_PING_PAYLOAD: [u8; 8] = [0x9, 0x2, 0xa, 0x3, 0x2, 0xe, 0x1, 0xf];

pub struct PingFrameCompressModel {
    flags: u8,
    payload: [u8; 8]
}

impl PingFrameCompressModel {
    pub fn new() -> Self {
        PingFrameCompressModel {
            flags: 0,
            payload: DEFAULT_PING_PAYLOAD
        }
    }

    pub fn set_acknowledge(&mut self) {
        self.flags |= FLAG_ACK;
    }

    pub fn set_ping_payload(&mut self, payload: [u8; 8]) {
        self.payload = payload;
    }
}

impl CompressibleHttpFrame for PingFrameCompressModel {
    fn get_length(&self) -> i32 {
        // a ping frame payload is always 8 octets
        8
    }

    fn get_frame_type(&self) -> FrameType {
        FrameType::Ping
    }

    fn get_flags(&self) -> u8 {
        self.flags
    }

    fn get_payload(self) -> Vec<u8> {
        self.payload.to_vec()
    }
}

pub struct PingFrame {
    payload: [u8; 8]
}

impl PingFrame {
    pub fn new(frame_header: &super::FrameHeader, frame: &mut IntoIter<u8>) -> Result<Self, error::HttpError> {
        // (6.7) PING frame with length field other than 8 is a connection error of type FRAME_SIZE_ERROR.
        if frame_header.length != 8 {
            return Err(error::HttpError::ConnectionError(
                error::ErrorCode::FrameSizeError,
                error::ErrorName::PingPayloadLength
            ));
        }

        Ok(PingFrame {
            payload: [
                frame.next().unwrap(),
                frame.next().unwrap(),
                frame.next().unwrap(),
                frame.next().unwrap(),
                frame.next().unwrap(),
                frame.next().unwrap(),
                frame.next().unwrap(),
                frame.next().unwrap()
            ]
        })
    }

    pub fn get_payload(&self) -> [u8; 8] {
        self.payload
    }
}

pub fn is_acknowledge(flags: u8) -> bool {
    flags & FLAG_ACK == FLAG_ACK
}
