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

pub trait HttpFrame {
    fn get_length(&self) -> i32;

    fn get_frame_type() -> u8;

    fn get_flags(&self) -> u8;

    fn get_payload(self) -> Vec<u8>;
}

pub fn build_frame<T: HttpFrame>(frame: T) -> Vec<u8> {
    let mut result = Vec::new();

    let length = frame.get_length();

    assert_eq!((length >> 24) as u8, 0, "frame size error");
    result.push((length >> 16) as u8);
    result.push((length >> 8) as u8);
    result.push(length as u8);

    // TODO build the rest of the frame

    result
}
