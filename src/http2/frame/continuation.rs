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

const FLAG_END_HEADERS: u8 = 0x4;

pub struct ContinuationFrameCompressModel {
    flags: u8,
    header_block_fragment: Vec<u8>
}

impl ContinuationFrameCompressModel {
    pub fn new(header_block_fragment: Vec<u8>) -> Self {
        ContinuationFrameCompressModel {
            flags: 0,
            header_block_fragment: header_block_fragment
        }
    }

    pub fn set_end_headers(&mut self) {
        self.flags |= FLAG_END_HEADERS;
    }
}

impl CompressibleHttpFrame for ContinuationFrameCompressModel {
    fn get_length(&self) -> i32 {
        self.header_block_fragment.len() as i32
    }

    fn get_frame_type(&self) -> FrameType {
        FrameType::Continuation
    }

    fn get_flags(&self) -> u8 {
        self.flags
    }

    fn get_payload(self) -> Vec<u8> {
        self.header_block_fragment
    }
}

pub struct ContinuationFrame {
    end_headers: bool,
    header_block_fragment: Vec<u8>
}

impl ContinuationFrame {
    pub fn new(frame_header: &super::FrameHeader, frame: &mut IntoIter<u8>) -> Self {
        let mut header_block_fragment = Vec::new();

        for _ in 0..frame_header.length {
            header_block_fragment.push(frame.next().unwrap());
        }

        ContinuationFrame {
            end_headers: frame_header.flags & FLAG_END_HEADERS == FLAG_END_HEADERS,
            header_block_fragment: header_block_fragment
        }
    }

    pub fn get_header_block_fragment(&self) -> &[u8] {
        self.header_block_fragment.as_slice()
    }

    pub fn is_end_headers(&self) -> bool {
        self.end_headers
    }
}

pub fn is_end_headers(flags: u8) -> bool {
    flags & FLAG_END_HEADERS == FLAG_END_HEADERS
}
