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

const WINDOW_SIZE_INCREMENT_BIT_MASK: u8 = 0x80;

#[derive(Debug)]
pub struct WindowUpdateFrameCompressModel {
    window_size_increment: u32
}

impl WindowUpdateFrameCompressModel {
    pub fn new(window_size_increment: u32) -> Self {
        WindowUpdateFrameCompressModel {
            window_size_increment: window_size_increment
        }
    }
}

impl CompressibleHttpFrame for WindowUpdateFrameCompressModel {
    fn get_length(&self) -> i32 {
        // 4 octets for the 32 bits in the window size increment
        4
    }

    fn get_frame_type(&self) -> FrameType {
        FrameType::WindowUpdate
    }

    fn get_flags(&self) -> u8 {
        // this frame doesn't define any flags
        0
    }

    fn get_payload(self: Box<Self>) -> Vec<u8> {
        let mut result = Vec::new();

        // include the window size increment
        let window_size_increment_first_octet = (self.window_size_increment >> 24) as u8;
        // TODO handle error
        assert_eq!(0, window_size_increment_first_octet & WINDOW_SIZE_INCREMENT_BIT_MASK);
        result.push(window_size_increment_first_octet & !WINDOW_SIZE_INCREMENT_BIT_MASK);
        result.push((self.window_size_increment >> 16) as u8);
        result.push((self.window_size_increment >> 8) as u8);
        result.push(self.window_size_increment as u8);

        result
    }
}

pub struct WindowUpdateFrame {
    window_size_increment: u32
}

impl WindowUpdateFrame {
    pub fn new_conn(frame_header: &super::FrameHeader, frame: &mut IntoIter<u8>) -> Result<Self, error::HttpError> {
        if frame_header.length != 4 {
            return Err(error::HttpError::ConnectionError(
                error::ErrorCode::FrameSizeError,
                error::ErrorName::InvalidFrameLengthForConnectionWindowUpdateFrame
            ));
        }

        let window_size_increment_first_octet = frame.next().unwrap();

        assert_eq!(0, window_size_increment_first_octet & WINDOW_SIZE_INCREMENT_BIT_MASK);

        Ok(WindowUpdateFrame {
            window_size_increment: 
                (((window_size_increment_first_octet & !WINDOW_SIZE_INCREMENT_BIT_MASK) as u32) << 24) +
                ((frame.next().unwrap() as u32) << 16) +
                ((frame.next().unwrap() as u32) << 8) +
                (frame.next().unwrap() as u32)
        })
    }

    pub fn new_stream(frame_header: &super::StreamFrameHeader, frame: &mut IntoIter<u8>) -> Result<Self, error::HttpError> {
        if frame_header.length != 4 {
            return Err(error::HttpError::ConnectionError(
                error::ErrorCode::FrameSizeError,
                error::ErrorName::InvalidFrameLengthForConnectionWindowUpdateFrame
            ));
        }

        let window_size_increment_first_octet = frame.next().unwrap();

        assert_eq!(0, window_size_increment_first_octet & WINDOW_SIZE_INCREMENT_BIT_MASK);

        Ok(WindowUpdateFrame {
            window_size_increment: 
                (((window_size_increment_first_octet & !WINDOW_SIZE_INCREMENT_BIT_MASK) as u32) << 24) +
                ((frame.next().unwrap() as u32) << 16) +
                ((frame.next().unwrap() as u32) << 8) +
                (frame.next().unwrap() as u32)
        })
    }

    pub fn get_window_size_increment(&self) -> u32 {
        self.window_size_increment
    }
}
