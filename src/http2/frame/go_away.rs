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

use super::CompressibleHttpFrame;

const GOAWAY_FRAME_TYPE: u8 = 0x7;

const LAST_STREAM_IDENTIFIER_BIT_MASK: u8 = 0x80;

// std
use std::vec::IntoIter;

// osmium
use http2::error;

pub struct GoAwayFrameCompressModel {
    last_stream_identifier: u32,
    error_code: error::ErrorCode,
    additional_debug_data: Vec<u8>
}

impl GoAwayFrameCompressModel {
    pub fn new() -> Self {
        GoAwayFrameCompressModel {
            last_stream_identifier: 0,
            error_code: error::ErrorCode::NoError,
            additional_debug_data: Vec::new()
        }
    }
}

impl CompressibleHttpFrame for GoAwayFrameCompressModel {
    fn get_length(&self) -> i32 {
        // 4 for the last stream identifier, 4 for the error code and the length of
        // the additional debug data
        8 + self.additional_debug_data.len() as i32
    }

    fn get_frame_type(&self) -> u8 {
        GOAWAY_FRAME_TYPE
    }

    fn get_flags(&self) -> u8 {
        // this frame doesn't define any flags
        0
    }

    fn get_payload(self) -> Vec<u8> {
        let mut result = Vec::new();

        // include the last stream identifier
        let last_stream_identifier_first_octet = (self.last_stream_identifier >> 24) as u8;
        // TODO handle error
        assert_eq!(0, last_stream_identifier_first_octet & LAST_STREAM_IDENTIFIER_BIT_MASK);
        result.push(last_stream_identifier_first_octet & !LAST_STREAM_IDENTIFIER_BIT_MASK);
        result.push((self.last_stream_identifier >> 16) as u8);
        result.push((self.last_stream_identifier >> 8) as u8);
        result.push(self.last_stream_identifier as u8);

        // include the error code
        let error_code = self.error_code as u32;
        result.push((error_code >> 24) as u8);
        result.push((error_code >> 16) as u8);
        result.push((error_code >> 8) as u8);
        result.push(error_code as u8);        

        // include any additional debug data
        result.extend(self.additional_debug_data);
        
        result
    }
}

pub struct GoAwayFrame {
    last_stream_identifier: u32,
    error_code: error::ErrorCode,
    additional_debug_data: Vec<u8>
}

impl GoAwayFrame {
    pub fn new(frame_header: &super::FrameHeader, frame: &mut IntoIter<u8>) -> Self {
        let last_stream_identifier_first_octet = frame.next().unwrap();

        assert_eq!(0, last_stream_identifier_first_octet & LAST_STREAM_IDENTIFIER_BIT_MASK);

        let last_stream_identifier = 
            ((last_stream_identifier_first_octet & !LAST_STREAM_IDENTIFIER_BIT_MASK) as u32) << 24 +
            (frame.next().unwrap() as u32) << 16 +
            (frame.next().unwrap() as u32) << 8 +
            frame.next().unwrap() as u32;

        let error_code = 
            (frame.next().unwrap() as u32) << 24 +
            (frame.next().unwrap() as u32) << 16 +
            (frame.next().unwrap() as u32) << 8 +
            frame.next().unwrap() as u32;

        let mut opt_error_code = error::to_error_code(error_code);

        // The spec says that an unrecognised error code must not trigger 'any' special 
        // behaviour but we do have to provide an enum constant... (see Section 7)
        if opt_error_code.is_none() {
            // the error code was not recognised. It should not be treated specially, but may
            // be treated as an internal error
            opt_error_code = Some(error::ErrorCode::InternalError);
        }

        let mut additional_debug_data = Vec::new();
        for _ in 8..frame_header.length {
            additional_debug_data.push(frame.next().unwrap());
        }

        GoAwayFrame {
            last_stream_identifier: last_stream_identifier,
            error_code: opt_error_code.unwrap(),
            additional_debug_data: additional_debug_data
        }
    }

    pub fn get_last_stream_identifier(&self) -> u32 {
        self.last_stream_identifier
    }

    pub fn get_error_code(&self) -> &error::ErrorCode {
        &self.error_code
    }

    pub fn get_additional_debug_data(&self) -> &[u8] {
        self.additional_debug_data.as_slice()
    }
}
