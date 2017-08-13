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

const STREAM_DEPENDENCY_EXCLUSIVE_BIT_MASK: u8 = 0x80;

pub struct PriorityFrameCompressModel {
    stream_dependency: u32,
    stream_dependency_exclusive: bool,
    weight: u8
}

impl PriorityFrameCompressModel {
    pub fn new(stream_dependency: u32, weight: u8, exclusive: bool) -> Self {
        PriorityFrameCompressModel {
            stream_dependency: stream_dependency,
            stream_dependency_exclusive: exclusive,
            weight: weight
        }
    }
}

impl CompressibleHttpFrame for PriorityFrameCompressModel {
    fn get_length(&self) -> i32 {
        // 4 for the dependency and 1 for the weight
        5
    }

    fn get_frame_type(&self) -> FrameType {
        FrameType::Priority
    }

    fn get_flags(&self) -> u8 {
        // this frame doesn't define any flags
        0
    }

    fn get_payload(self) -> Vec<u8> {
        let mut result = Vec::new();

        // include the stream dependency
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
        
        result
    }
}

pub struct PriorityFrame {
    exclusive: bool,
    stream_dependency: u32,
    weight: u8
}

impl PriorityFrame {
    pub fn new(frame_header: &super::FrameHeader, frame: &mut IntoIter<u8>) -> Self {
        // TODO handle error
        assert_eq!(5, frame_header.length);

        let stream_dependency_first_octet = frame.next().unwrap();

        PriorityFrame {
            exclusive: stream_dependency_first_octet & STREAM_DEPENDENCY_EXCLUSIVE_BIT_MASK == STREAM_DEPENDENCY_EXCLUSIVE_BIT_MASK,
            stream_dependency: 
                ((stream_dependency_first_octet & !STREAM_DEPENDENCY_EXCLUSIVE_BIT_MASK) as u32) << 24 +
                (frame.next().unwrap() as u32) << 16 +
                (frame.next().unwrap() as u32) << 8 +
                (frame.next().unwrap() as u32),
            weight: frame.next().unwrap()
        }
    }

    pub fn get_stream_dependency(&self) -> u32 {
        self.stream_dependency
    }

    pub fn get_weight(&self) -> u8 {
        self.weight
    }

    pub fn is_exclusive(&self) -> bool {
        self.exclusive
    }
}
