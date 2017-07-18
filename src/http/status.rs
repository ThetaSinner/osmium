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
pub enum HttpStatus {
    Ok,
    BadRequest,
    NotFound,
    Custom(String, String)
}

impl fmt::Display for HttpStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            HttpStatus::Ok => write!(f, "200 OK"),
            HttpStatus::BadRequest => write!(f, "400 Bad Request"),
            HttpStatus::NotFound => write!(f, "404 Not Found"),
            HttpStatus::Custom(ref code, ref phrase) => write!(f, "{} {}", code, phrase)
        }
    }
}