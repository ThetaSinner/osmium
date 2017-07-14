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

// std
use std::fmt;

#[derive(Debug)]
pub enum HttpVersion {
    Http10,
    Http11,
    Http2,
}

impl From<u8> for HttpVersion {
    fn from(version: u8) -> HttpVersion {
        match version {
            0 => HttpVersion::Http10,
            1 => HttpVersion::Http11,
            // TODO httparse errors if the version is invalid so this line should not run right now
            // but needs to be changed if the version might not match.
            _ => panic!("Invalid http version")
        }
    }
}

impl fmt::Display for HttpVersion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            HttpVersion::Http10 => write!(f, "1.0"),
            HttpVersion::Http11 => write!(f, "1.1"),
            HttpVersion::Http2 => write!(f, "2")
        }
    }
}
