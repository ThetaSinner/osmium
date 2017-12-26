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

// regex
use regex::Regex;

// osmium
use http2::header;
use http2::hpack::context as hpack_context;
use http2::hpack::unpack as hpack_unpack;
use http2::error;

#[derive(Debug)]
pub struct StreamRequest {
    pub headers: header::Headers,
    pub payload: Option<Vec<u8>>,
    pub trailer_headers: Option<header::Headers>
}

impl StreamRequest {
    pub fn new() -> Self {
        StreamRequest {
            headers: header::Headers::new(),
            payload: None,
            trailer_headers: None
        }
    }

    pub fn process_temp_header_block(&mut self, temp_header_block: &[u8], hpack_recv_context: &mut hpack_context::RecvContext) -> Option<error::HttpError> {
        let mut decoded = hpack_unpack::UnpackedHeaders::<header::Header>::new();
        hpack_unpack::unpack(temp_header_block, hpack_recv_context, &mut decoded);

        // TODO can the header block be empty? because that will break the logic below.

        if self.headers.is_empty() {
            // If no request headers have been received then these are the request headers.
            match hpack_to_http2_headers(decoded.headers) {
                Ok(headers) => {
                    self.headers = headers;
                },
                Err(e) => return Some(e)
            }
        }
        else if self.trailer_headers.is_none() {
            // If no trailer headers have been received then these are the tailer headers.
            match hpack_to_http2_headers(decoded.headers) {
                Ok(headers) => {
                    self.trailer_headers = Some(headers);
                },
                Err(e) => return Some(e)
            }
        }
        else {
            // TODO handle error. We have received all the header blocks we were expecting, but received
            // a request to process another.
            panic!("unexpected header block");
        }

        None
    }
}

fn hpack_to_http2_headers(hpack_headers: Vec<header::Header>) -> Result<header::Headers, error::HttpError> {
    let mut headers = header::Headers::new();

    // TODO move to setup
    let header_name_regex = Regex::new(r"^:?[!#$%&'*+\-.^_`|~0-9a-z]+$").unwrap();

    // TODO this should be a move and not need mut?
    for mut header in hpack_headers.into_iter() {
        let new_name = match header.name {
            header::HeaderName::CustomHeader(name) => {
                // TODO this could be better, seeing as is has to be done many times
                if !header_name_regex.is_match(String::from(name.clone()).as_str()) {
                    error!("Rejecting header because of bad name {:?}", name);
                    return Err(error::HttpError::StreamError(
                        error::ErrorCode::ProtocolError,
                        error::ErrorName::NonLowerCaseHeaderNameIsRejectedAsMalformed
                    ));
                }

                name.as_str().into()
            },
            _ => {
                panic!("incorrect use of hpack");
            }
        };

        header.name = new_name;
        headers.push_header(header);
    }

    Ok(headers)
}
