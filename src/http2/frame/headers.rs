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

const HEADERS_FRAME_TYPE: u8 = 0x1;

const FLAG_END_STREAM: u8 = 0x1;
const FLAG_END_HEADERS: u8 = 0x4;
const FLAG_PADDED: u8 = 0x8;
const FLAG_PRIORITY: u8 = 0x20;

const STREAM_DEPENDENCY_EXCLUSIVE_BIT_MASK: u8 = 0x80;

// std
use std::vec::IntoIter;

pub struct HeadersFrameCompressModel {
    flags: u8,
    pad_length: u8,
    stream_dependency: u32,
    stream_dependency_exclusive: bool,
    weight: u8,
    header_block_fragment: Vec<u8>
}

impl HeadersFrameCompressModel {
    pub fn new(end_stream: bool, end_headers: bool) -> Self {
        let mut flags = 0;
        if end_stream {
            flags |= FLAG_END_STREAM;
        }
        if end_headers {
            flags |= FLAG_END_HEADERS;
        }

        HeadersFrameCompressModel {
            flags: flags,
            pad_length: 0,
            stream_dependency: 0,
            stream_dependency_exclusive: false,
            weight: 0,
            header_block_fragment: Vec::new()
        }
    }

    pub fn set_pad_length(&mut self, pad_length: u8) {
        self.pad_length = pad_length;

        self.flags |= FLAG_PADDED;
    }

    pub fn set_dependency(&mut self, stream_dependency: u32, weight: u8, exclusive: bool) {
        self.stream_dependency = stream_dependency;
        self.weight = weight;
        self.stream_dependency_exclusive = exclusive;

        self.flags |= FLAG_PRIORITY;
    }

    pub fn set_header_block_fragment(&mut self, header_block_fragment: Vec<u8>) {
        self.header_block_fragment = header_block_fragment;
    }
}

impl CompressibleHttpFrame for HeadersFrameCompressModel {
    fn get_length(&self) -> i32 {
        let mut length = self.header_block_fragment.len();
        
        if self.flags & FLAG_PADDED == FLAG_PADDED {
            length += 1 + self.pad_length as usize;
        }
        
        if self.flags & FLAG_PRIORITY == FLAG_PRIORITY {
            // 4 octets for the stream dependency (1 bit exclusivity flag and 31 bits for the dependency)
            // 1 octet for the weight
            length += 5;
        }

        length as i32
    }

    fn get_frame_type(&self) -> u8 {
        HEADERS_FRAME_TYPE
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

        // include the stream dependency
        if self.flags & FLAG_PRIORITY == FLAG_PRIORITY {
            let mut stream_dependency_first_octet = (self.stream_dependency >> 24) as u8;
            // TODO
            assert_eq!(0, stream_dependency_first_octet & STREAM_DEPENDENCY_EXCLUSIVE_BIT_MASK);
            if self.stream_dependency_exclusive {
                stream_dependency_first_octet |= STREAM_DEPENDENCY_EXCLUSIVE_BIT_MASK;
            }
            result.push(stream_dependency_first_octet);
            result.push((self.stream_dependency >> 16) as u8);
            result.push((self.stream_dependency >> 8) as u8);
            result.push(self.stream_dependency as u8);

            // include the weight
            // TODO
            assert!(1 <= self.weight);
            result.push(self.weight);
        }
    
        // include the header block fragment
        result.extend(self.header_block_fragment);

        // TODO there has to be a better way to express this.
        for _ in 0..self.pad_length {
            result.push(0);
        }

        result
    }
}

pub struct Priority {
    pub exclusive: bool,
    pub stream_dependency: u32,
    pub weight: u8
}

pub struct HeaderFrame {
    priority: Option<Priority>,
    header_block_fragment: Vec<u8>,
    end_stream: bool,
    end_headers: bool,
}

impl HeaderFrame {
    pub fn new(frame_header: &super::FrameHeader, frame: &mut IntoIter<u8>) -> Self {
        let mut read_length = 0;

        let pad_length = if frame_header.flags & FLAG_PADDED == FLAG_PADDED {
            read_length += 1;
            frame.next().unwrap()
        }
        else {
            0
        };

        let priority = if frame_header.flags & FLAG_PRIORITY == FLAG_PRIORITY {
            read_length += 5;

            let stream_dependency_first_octet = frame.next().unwrap();

            Some(Priority {
                exclusive: stream_dependency_first_octet & STREAM_DEPENDENCY_EXCLUSIVE_BIT_MASK == STREAM_DEPENDENCY_EXCLUSIVE_BIT_MASK,
                stream_dependency: 
                    (stream_dependency_first_octet as u32) << 24 +
                    (frame.next().unwrap() as u32) << 16 +
                    (frame.next().unwrap() as u32) << 8 +
                    (frame.next().unwrap() as u32),
                weight: frame.next().unwrap()
            })
        }
        else {
            None
        };

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

        HeaderFrame {
            priority: priority,
            header_block_fragment: header_block_fragment,
            end_stream: frame_header.flags & FLAG_END_STREAM == FLAG_END_STREAM,
            end_headers: frame_header.flags & FLAG_END_HEADERS == FLAG_END_HEADERS
        }
    }

    pub fn get_priority(&self) -> &Option<Priority> {
        &self.priority
    }

    pub fn get_header_block_fragment(&self) -> &[u8] {
        self.header_block_fragment.as_slice()
    }

    pub fn is_end_stream(&self) -> bool {
        self.end_stream
    }

    pub fn is_end_headers(&self) -> bool {
        self.end_headers
    }
}
