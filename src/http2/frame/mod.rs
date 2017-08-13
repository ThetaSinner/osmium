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
pub mod priority;
pub mod reset_stream;
pub mod settings;
pub mod push_promise;
pub mod ping;
pub mod go_away;
pub mod window_update;
pub mod continuation;

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

    fn get_frame_type(&self) -> FrameType;

    fn get_flags(&self) -> u8;

    fn get_payload(self) -> Vec<u8>;
}

#[derive(Debug)]
pub struct FrameHeader {
    pub length: u32,
    pub frame_type: Option<FrameType>,
    pub flags: u8,
    pub stream_id: u32
}

#[derive(Debug)]
pub enum FrameType {
    Data,
    Headers,
    Priority,
    ResetStream,
    Settings,
    PushPromise,
    Ping,
    GoAway,
    WindowUpdate,
    Continuation
}

impl From<FrameType> for u8 {
    fn from(frame_type: FrameType) -> u8 {
        match frame_type {
            FrameType::Data => 0x0,
            FrameType::Headers => 0x1,
            FrameType::Priority => 0x2,
            FrameType::ResetStream => 0x3,
            FrameType::Settings => 0x4,
            FrameType::PushPromise => 0x5,
            FrameType::Ping => 0x6,
            FrameType::GoAway => 0x7,
            FrameType::WindowUpdate => 0x8,
            FrameType::Continuation => 0x9
        }
    }
}

pub fn to_frame_type(frame_type: u8) -> Option<FrameType> {
    match frame_type {
        0x0 => Some(FrameType::Data),
        0x1 => Some(FrameType::Headers),
        0x2 => Some(FrameType::Priority),
        0x3 => Some(FrameType::ResetStream),
        0x4 => Some(FrameType::Settings),
        0x5 => Some(FrameType::PushPromise),
        0x6 => Some(FrameType::Ping),
        0x7 => Some(FrameType::GoAway),
        0x8 => Some(FrameType::WindowUpdate),
        0x9 => Some(FrameType::Continuation),
        _ => None
    }
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

    result.push(frame.get_frame_type() as u8);
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
        frame_type: to_frame_type(frame_iter.next().unwrap()),
        flags: frame_iter.next().unwrap(),
        stream_id:
            ((STREAM_IDENTIFIER_RESERVED_BIT_MASK & frame_iter.next().unwrap()) as u32) << 24 +
            (frame_iter.next().unwrap() as u32) << 16 +
            (frame_iter.next().unwrap() as u32) << 8 +
            (frame_iter.next().unwrap() as u32)
    };

    (frame_header, frame_iter)
}
