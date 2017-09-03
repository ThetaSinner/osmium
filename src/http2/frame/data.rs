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

const FLAG_END_STREAM: u8 = 0x1;
const FLAG_PADDED: u8 = 0x8;

pub struct DataFrameCompressModel {
    flags: u8,
    pad_length: u8,
    payload: Vec<u8>
}

impl DataFrameCompressModel {
    pub fn new(end_stream: bool) -> Self {
        DataFrameCompressModel {
            flags: if end_stream {
                FLAG_END_STREAM
            }
            else {
                0
            },
            pad_length: 0,
            payload: Vec::new()
        }
    }

    // An input of 0 for pad_length is valid, and allows the payload length to 
    // be increased by 1. Therefore, a non-zero check against pad_length is 
    // not valid to check if padding should be included when this frame is 
    // compressed. Use the flag instead.
    pub fn set_pad_length(&mut self, pad_length: u8) {
        self.pad_length = pad_length;

        // update the padded flag.
        self.flags |= FLAG_PADDED;
    }

    pub fn set_end_stream(&mut self) {
        self.flags |= FLAG_END_STREAM;
    }

    pub fn set_payload(&mut self, payload: Vec<u8>) {
        self.payload = payload;
    }
}

impl CompressibleHttpFrame for DataFrameCompressModel {
    fn get_length(&self) -> i32 {
        // The entire data frame payload is included in flow control, including
        // the pad length and padding fields if present.
        if self.flags & FLAG_PADDED == FLAG_PADDED {
            (self.payload.len() + 1 + self.pad_length as usize) as i32
        }
        else {
            self.payload.len() as i32
        }
    }

    fn get_frame_type(&self) -> FrameType {
        FrameType::Data
    }

    fn get_flags(&self) -> u8 {
        self.flags
    }

    fn get_payload(self: Box<Self>) -> Vec<u8> {
        let pad_length = self.pad_length;
        let mut result = Vec::new();
        if self.flags & FLAG_PADDED == FLAG_PADDED {
            result.push(pad_length)
        }
        result.extend(self.payload);

        // TODO there has to be a better way to express this.
        for _ in 0..pad_length {
            result.push(0);
        }

        result
    }
}

pub struct DataFrame {
    payload: String,
    end_stream: bool
}

impl DataFrame {
    pub fn new(frame_header: &super::StreamFrameHeader, frame: &mut IntoIter<u8>) -> DataFrame {
        let pad_length = if frame_header.flags & FLAG_PADDED == FLAG_PADDED {
            frame.next().unwrap()
        }
        else {
            0
        };

        let mut payload = Vec::new();
        for _ in 0..frame_header.length {
            payload.push(frame.next().unwrap());
        }

        for _ in 0..pad_length {
            frame.next().unwrap();
        }

        DataFrame {
            payload: String::from_utf8(payload).unwrap(),
            end_stream: frame_header.flags & FLAG_END_STREAM == FLAG_END_STREAM
        }
    }

    pub fn get_payload(&self) -> &str {
        &self.payload
    }

    pub fn is_end_stream(&self) -> bool {
        self.end_stream
    }
}
