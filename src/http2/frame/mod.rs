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

pub mod data;
pub mod headers;

// Denoted 'R' in http2 Section 4.1
const STREAM_IDENTIFIER_RESERVED_BIT_MASK: u8 = !0x80;

// std
use std::vec::IntoIter;

// osmium
pub use self::data::DataFrame;

// TODO the template layed out in this file and the data file will hopefully work
// with the http2 transport mechanism, but in order to see the extent to which 
// this code needs to handle streaming of partial frames etc there needs to be data 
// to push into this module. Thus I'm leaving this for now and moving onto the 
// transport code.

pub trait CompressibleHttpFrame {
    fn get_length(&self) -> i32;

    fn get_frame_type(&self) -> u8;

    fn get_flags(&self) -> u8;

    fn get_payload(self) -> Vec<u8>;
}

pub struct FrameHeader {
    pub length: u32,
    pub frame_type: u8,
    pub flags: u8,
    pub stream_id: u32
}

pub fn compress_frame<T>(frame: T, stream_id: u32) -> Vec<u8>
    where T : CompressibleHttpFrame 
{
    let mut result = Vec::new();

    let length = frame.get_length();

    assert_eq!((length >> 24) as u8, 0, "frame size error");
    result.push((length >> 16) as u8);
    result.push((length >> 8) as u8);
    result.push(length as u8);

    result.push(frame.get_frame_type());
    result.push(frame.get_flags());

    result.push(STREAM_IDENTIFIER_RESERVED_BIT_MASK & (stream_id >> 24) as u8);
    result.push((stream_id >> 16) as u8);
    result.push((stream_id >> 8) as u8);
    result.push(stream_id as u8);

    result.extend(frame.get_payload());

    result
}

pub fn decompress_frame(frame: Vec<u8>) -> (FrameHeader, IntoIter<u8>) {
    // a frame should always have a header which is 9 octets long.
    assert!(frame.len() >= 9);

    let mut frame_iter = frame.into_iter();

    let frame_header = FrameHeader {
        length:
            (frame_iter.next().unwrap() as u32) << 16 +
            (frame_iter.next().unwrap() as u32) << 8 +
            (frame_iter.next().unwrap() as u32),
        frame_type: frame_iter.next().unwrap(),
        flags: frame_iter.next().unwrap(),
        stream_id:
            ((STREAM_IDENTIFIER_RESERVED_BIT_MASK & frame_iter.next().unwrap()) as u32) << 24 +
            (frame_iter.next().unwrap() as u32) << 16 +
            (frame_iter.next().unwrap() as u32) << 8 +
            (frame_iter.next().unwrap() as u32)
    };

    (frame_header, frame_iter)
}
