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
use std::fmt;
use std::slice;

#[derive(Clone, Debug)]
pub enum HeaderName {
    ContentLength,
    Host,
    Accept,
    CustomHeader(String)
}

#[derive(Debug)]
pub enum HeaderValue {
    Str(String),
    Num(i32)
}

#[derive(Debug)]
pub struct Header(HeaderName, HeaderValue);

#[derive(Debug)]
pub struct Headers {
    lookup_map: HashMap<String, usize>,
    headers: Vec<Header>
}

impl Headers {
    pub fn new() -> Self {
        Headers {
            lookup_map: HashMap::new(),
            headers: Vec::new()
        }
    }

    pub fn add_header(&mut self, header: Header) {
        self.lookup_map.insert(String::from(header.0.clone()), self.headers.len());
        self.headers.push(header);
    }

    pub fn add(&mut self, name: HeaderName, value: HeaderValue) {
        self.lookup_map.insert(String::from(name.clone()), self.headers.len());
        self.headers.push(Header(name, value));
    }

    // pub fn get(&self, name: HeaderName) -> Option<&HeaderValue> {
    //     let opt_index = self.lookup_map.get(&String::from(name));

    //     if let Some(&index) = opt_index {
    //         self.headers.get(index)
    //     }
    //     else {
    //         None
    //     }
    // }

    pub fn iter(&self) -> slice::Iter<Header> {
        self.headers.iter()
    }
}

// Convert `HeaderName` enum values to string for serialisation 
// and so that the enum type can be pseudo-used as a hash key.
impl From<HeaderName> for String {
    fn from(name: HeaderName) -> Self {
        match name {
            HeaderName::ContentLength => String::from("Content-Length"),
            HeaderName::Host => String::from("Host"),
            HeaderName::Accept => String::from("Accept"),
            HeaderName::CustomHeader(v) => v
        }
    }
}

// Convert strings to `HeaderName` for http request deserialisation.
impl<'a> From<&'a str> for HeaderName {
    fn from(name: &str) -> Self {
        match name {
            "Content-Length" => HeaderName::ContentLength,
            "Host" => HeaderName::Host,
            "Accept" => HeaderName::Accept,
            _ => {
                info!("Missing header conversion for [{}]. Will treat as custom header.", name);
                HeaderName::CustomHeader(String::from(name))
            }
        }
    }
}

impl fmt::Display for HeaderValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            HeaderValue::Str(ref s) => write!(f, "{}", s),
            HeaderValue::Num(ref n) => write!(f, "{}", n)
        }
    }
}
