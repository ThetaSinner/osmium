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
use std::slice;

#[derive(Clone, PartialEq, Debug)]
pub enum HeaderName {
    PseudoPath,
    PseudoMethod,
    PseudoScheme,
    PseudoAuthority,
    ContentLength,
    CacheControl,
    Host,
    Accept,
    Date,
    CustomHeader(String)
}

#[derive(Clone, PartialEq, Debug)]
pub enum HeaderValue {
    Str(String),
    Num(i32)
}

#[derive(Debug)]
pub struct Header {
    pub name: HeaderName, 
    pub value: HeaderValue,
    allow_compression: bool
}

impl Header {
    pub fn new(name: HeaderName, value: HeaderValue) -> Self {
        Header {
            name: name,
            value: value,
            allow_compression: true
        }
    }

    pub fn set_allow_compression(&mut self, allow_compression: bool) {
        self.allow_compression = allow_compression;
    }

    pub fn is_allow_compression(&self) -> bool {
        self.allow_compression
    }
}

#[derive(Debug)]
pub struct Headers {
    headers: Vec<Header>
}

impl Headers {
    pub fn new() -> Self {
        Headers {
            headers: Vec::new()
        }
    }

    pub fn push_header(&mut self, header: Header) {
        self.headers.push(header);
    }

    pub fn push(&mut self, name: HeaderName, value: HeaderValue) {
        self.headers.push(Header::new(name, value));
    }

    pub fn iter(&self) -> slice::Iter<Header> {
        self.headers.iter()
    }

    pub fn len(&self) -> usize {
        self.headers.len()
    }
}

// Convert `HeaderName` enum values to string for serialisation 
// and so that the enum type can be pseudo-used as a hash key.
impl From<HeaderName> for String {
    fn from(name: HeaderName) -> Self {
        match name {
            HeaderName::PseudoPath => String::from(":path"),
            HeaderName::PseudoMethod => String::from(":method"),
            HeaderName::PseudoScheme => String::from(":scheme"),
            HeaderName::PseudoAuthority => String::from(":authority"),
            HeaderName::ContentLength => String::from("Content-Length"),
            HeaderName::CacheControl => String::from("Cache-Control"),
            HeaderName::Host => String::from("Host"),
            HeaderName::Accept => String::from("Accept"),
            HeaderName::Date => String::from("Date"),
            HeaderName::CustomHeader(v) => v
        }
    }
}

// TODO according to MDN (and I assume the http spec) header names are case insensitive, so any matching
// needs to not assume case

// Convert strings to `HeaderName` for http request deserialisation.
impl<'a> From<&'a str> for HeaderName {
    fn from(name: &str) -> Self {
        match name {
            ":path" => HeaderName::PseudoPath,
            ":method" => HeaderName::PseudoMethod,
            ":scheme" => HeaderName::PseudoScheme,
            ":authority" => HeaderName::PseudoAuthority,
            "Content-Length" => HeaderName::ContentLength,
            "Cache-Control" => HeaderName::CacheControl,
            "Host" => HeaderName::Host,
            "Accept" => HeaderName::Accept,
            "Date" => HeaderName::Date,
            _ => {
                info!("Missing header conversion for [{}]. Will treat as custom header.", name);
                HeaderName::CustomHeader(String::from(name))
            }
        }
    }
}

impl From<HeaderValue> for String {
    fn from(value: HeaderValue) -> Self {
        match value {
            HeaderValue::Str(v) => v,
            HeaderValue::Num(v) => format!("{}", v).to_owned()
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
