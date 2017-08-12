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

const PUSH_PROMISE_FRAME_TYPE: u8 = 0x1;

const FLAG_END_HEADERS: u8 = 0x4;
const FLAG_PADDED: u8 = 0x8;

const PROMISED_STREAM_IDENTIFIER_RESERVED_BIT_MASK: u8 = 0x80;

// std
use std::vec::IntoIter;

pub struct PushPromiseFrameCompressModel {
    flags: u8,
    pad_length: u8,
    promised_stream_identifier: u32,
    header_block_fragment: Vec<u8>
}

impl PushPromiseFrameCompressModel {
    pub fn new(end_headers: bool) -> Self {
        let flags = if end_headers {
            FLAG_END_HEADERS
        }
        else {
            0
        };

        PushPromiseFrameCompressModel {
            flags: flags,
            pad_length: 0,
            promised_stream_identifier: 0,
            header_block_fragment: Vec::new()
        }
    }

    pub fn set_pad_length(&mut self, pad_length: u8) {
        self.pad_length = pad_length;

        self.flags |= FLAG_PADDED;
    }

    pub fn set_promised_stream_identifier(&mut self, promised_stream_identifier: u32) {
        self.promised_stream_identifier = promised_stream_identifier;
    }

    pub fn set_header_block_fragment(&mut self, header_block_fragment: Vec<u8>) {
        self.header_block_fragment = header_block_fragment;
    }
}

impl CompressibleHttpFrame for PushPromiseFrameCompressModel {
    fn get_length(&self) -> i32 {
        // the 4 is for the promised stream identifier
        let mut length = 4 + self.header_block_fragment.len();
        
        if self.flags & FLAG_PADDED == FLAG_PADDED {
            length += 1 + self.pad_length as usize;
        }

        length as i32
    }

    fn get_frame_type(&self) -> u8 {
        PUSH_PROMISE_FRAME_TYPE
    }

    fn get_flags(&self) -> u8 {
        self.flags
    }

    fn get_payload(self) -> Vec<u8> {
        let mut result = Vec::new();

        // include the pad length if set
        if self.flags & FLAG_PADDED == FLAG_PADDED {
            result.push(self.pad_length)
        }

        // include the promised stream identifier
        let promised_stream_identifier_first_octet = (self.promised_stream_identifier >> 24) as u8;

        assert_eq!(0, promised_stream_identifier_first_octet & PROMISED_STREAM_IDENTIFIER_RESERVED_BIT_MASK);
        
        result.push(promised_stream_identifier_first_octet & !PROMISED_STREAM_IDENTIFIER_RESERVED_BIT_MASK);
        result.push((self.promised_stream_identifier >> 16) as u8);
        result.push((self.promised_stream_identifier >> 8) as u8);
        result.push(self.promised_stream_identifier as u8);

        // include the header block fragment
        result.extend(self.header_block_fragment);

        // TODO there has to be a better way to express this.
        for _ in 0..self.pad_length {
            result.push(0);
        }

        result
    }
}

pub struct PushPromiseFrame {
    promised_stream_identifier: u32,
    header_block_fragment: Vec<u8>,
    end_headers: bool,
}

impl PushPromiseFrame {
    pub fn new(frame_header: &super::FrameHeader, frame: &mut IntoIter<u8>) -> Self {
        let mut read_length = 0;

        let pad_length = if frame_header.flags & FLAG_PADDED == FLAG_PADDED {
            read_length += 1;
            frame.next().unwrap()
        }
        else {
            0
        };

        // read the promised stream identifier
        read_length += 4;
        let promised_stream_identifier_first_octet = frame.next().unwrap();
        // TODO handle error
        assert_eq!(0, promised_stream_identifier_first_octet & PROMISED_STREAM_IDENTIFIER_RESERVED_BIT_MASK);

        let promised_stream_identifier =
            (promised_stream_identifier_first_octet as u32) << 24 +
            (frame.next().unwrap() as u32) << 16 +
            (frame.next().unwrap() as u32) << 8 +
            frame.next().unwrap() as u32;

        let mut header_block_fragment = Vec::new();
        // the number of octets in the header fragment is the payload length without any header octets already read
        // and without the padding bits
        for _ in 0..(frame_header.length - read_length - pad_length as u32) {
            header_block_fragment.push(frame.next().unwrap());
            read_length += 1;
        }

        // TODO check the integrity of the header.
        assert_eq!(pad_length as u32, frame_header.length - read_length);

        for _ in 0..pad_length {
            frame.next().unwrap();
        }

        PushPromiseFrame {
            promised_stream_identifier: promised_stream_identifier,
            header_block_fragment: header_block_fragment,
            end_headers: frame_header.flags & FLAG_END_HEADERS == FLAG_END_HEADERS
        }
    }

    pub fn get_prromised_stream_identifier(&self) -> u32 {
        self.promised_stream_identifier
    }

    pub fn get_header_block_fragment(&self) -> &[u8] {
        self.header_block_fragment.as_slice()
    }

    pub fn is_end_headers(&self) -> bool {
        self.end_headers
    }
}
