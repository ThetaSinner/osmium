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
// along with Osmium.  If not, see <http://www.gnu.org/licenses/>.

use super::CompressibleHttpFrame;

const DATA_FRAME_TYPE: u8 = 0x0;

const FLAG_END_STREAM: u8 = 0x1;
const FLAG_PADDED: u8 = 0x8;

pub struct DataFrame {
    flags: u8,
    pad_length: u8,
    payload: Vec<u8>
}

impl DataFrame {
    pub fn new(end_stream: bool) -> Self {
        DataFrame {
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

    pub fn set_pad_length(&mut self, pad_length: u8) {
        self.pad_length = pad_length;

        // update the padded flag.
        if pad_length == 0 {
            self.flags |= FLAG_PADDED;
        }
        else {
            self.flags &= !FLAG_PADDED;
        }
    }

    pub fn set_payload(&mut self, payload: Vec<u8>) {
        self.payload = payload;
    }
}

impl CompressibleHttpFrame for DataFrame {
    fn get_length(&self) -> i32 {
        self.payload.len() as i32
    }

    fn get_frame_type(&self) -> u8 {
        DATA_FRAME_TYPE
    }

    fn get_flags(&self) -> u8 {
        self.flags
    }

    fn get_payload(self) -> Vec<u8> {
        let mut result = Vec::new();
        if self.pad_length != 0 {
            result.push(self.pad_length)
        }
        result.extend(self.payload);

        // TODO there has to be a better way to express this.
        for _ in 0..self.pad_length {
            result.push(0);
        }

        result
    }
}
