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
use std::io;

// tokio
use bytes::{Buf, IntoBuf, BytesMut};

// osmium
use http_version::HttpVersion;
use http::request::Request;
use http::header::Headers;

pub fn decode(buf: &mut BytesMut) -> io::Result<Option<Request>> {
    let len = buf.len();
    let t = buf.split_to(len);

    if len > 0 {
        Ok(Some(Request {
            version: HttpVersion::Http11,
            uri: "/".to_owned(),
            headers: Headers::new(),
            body: None
        }))
    }
    else {
        Ok(None)
    }
}
