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
use std::collections::HashMap;

#[derive(Debug)]
pub enum HeaderName {
    ContentLength
}

#[derive(Debug)]
pub enum HeaderValue {
    Str(String),
    Num(i32)
}

#[derive(Debug)]
pub struct Headers {
    headers: HashMap<String, HeaderValue>
}

impl Headers {
    pub fn new() -> Self {
        Headers {
            headers: HashMap::new()
        }
    }

    pub fn add(&mut self, name: HeaderName, value: HeaderValue) {
        self.headers.insert(String::from(name), value);
    }


}

impl From<HeaderName> for String {
    fn from(name: HeaderName) -> Self {
        match name {
            HeaderName::ContentLength => String::from("Content-Length")
        }
    }
}
