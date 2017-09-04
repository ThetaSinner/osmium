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

#[derive(Debug)]
pub struct ResetStreamFrameCompressModel {
    error_code: u32
}

impl ResetStreamFrameCompressModel {
    pub fn new(error_code: u32) -> Self {
        ResetStreamFrameCompressModel {
            error_code: error_code
        }
    }
}

impl CompressibleHttpFrame for ResetStreamFrameCompressModel {
    fn get_length(&self) -> i32 {
        // the 4 octets in the 32 bit octet.
        4
    }

    fn get_frame_type(&self) -> FrameType {
        FrameType::ResetStream
    }

    fn get_flags(&self) -> u8 {
        // this frame doesn't define any flags
        0
    }

    fn get_payload(self: Box<Self>) -> Vec<u8> {
        let mut result = Vec::new();

        result.push((self.error_code >> 24) as u8);
        result.push((self.error_code >> 16) as u8);
        result.push((self.error_code >> 8) as u8);
        result.push(self.error_code as u8);

        result
    }
}

pub struct ResetStreamFrame {
    error_code: u32
}

impl ResetStreamFrame {
    pub fn new(frame_header: &super::StreamFrameHeader, frame: &mut IntoIter<u8>) -> Self {
        // TODO handle error
        assert_eq!(4, frame_header.length);

        ResetStreamFrame {
            error_code:
                (frame.next().unwrap() as u32) << 24 +
                (frame.next().unwrap() as u32) << 16 +
                (frame.next().unwrap() as u32) << 8 +
                (frame.next().unwrap() as u32)
        }
    }

    pub fn get_error_code(&self) -> u32 {
        self.error_code
    }
}
