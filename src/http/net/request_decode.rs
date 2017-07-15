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
use http::header::{Headers, HeaderName, HeaderValue};
use httparse;

pub fn decode(buf: &mut BytesMut) -> io::Result<Option<Request>> {
    // TODO Not quite sure under what conditions this buffer is empty and how to handle that.
    // For now assume we don't want to process and further and end this decode.
    if buf.len() == 0 {
        return Ok(None)
    }

    let response = {
        let mut headers = [httparse::EMPTY_HEADER; 16];
        let mut req = httparse::Request::new(&mut headers);

        // The parse is done inside a block because of buffer ownership. Feel free to read the docs for `bytes`, `httparse` and `tokio_io` 
        let parsing = req.parse(buf).unwrap();
        if parsing.is_complete() {
            let version = if let Some(v) = req.version {
                HttpVersion::from(v)
            }
            else {
                // TODO default for now.
                HttpVersion::Http11
            };

            let uri = req.path.unwrap_or("/");

            let mut headers = Headers::new();
            for req_header in req.headers.iter() {
                let header_name = HeaderName::from(req_header.name);
                let val = String::from_utf8(req_header.value.to_vec()).unwrap();
                match header_name {
                    HeaderName::ContentLength => {
                        let val = val.parse::<i32>().unwrap();
                        headers.add(header_name, HeaderValue::Num(val));
                    },
                    _ => headers.add(header_name, HeaderValue::Str(val))
                }
            }

            info!("Request ok, proceeding.");

            Some(Request {
                version: version,
                uri: uri.to_owned(),
                headers: Headers::new(),
                body: None
            })
        }
        else {
            error!("Server does not support streamed requests yet.");
            None
        }
    };

    // Consume all data from the buffer.
    let len = buf.len();
    buf.split_to(len);

    Ok(response)
}
