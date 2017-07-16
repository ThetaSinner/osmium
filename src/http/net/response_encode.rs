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
use std::fmt::Write;

// tokio
use bytes::BytesMut;

// osmium
use http::response::Response;
use http::header::HeaderValue;

/// Convert a `Response` struct to http transport format
///
/// This is called from the http codec which is the tokio tranport format conversion.
pub fn encode(res: Response, buf: &mut BytesMut) {
    debug!("Encoding response [{:?}]", res);
    write!(buf, "HTTP/{} {}", res.version, res.status).unwrap();
    let ref headers = res.headers;
    for (name, value) in headers.iter() {
        write!(buf, "\r\n{}: {}", name, value).unwrap();
    }

    write!(buf, "\r\n\r\n");

    if let Some(body) = res.body {
        write!(buf, "{}", body).unwrap();
    }

    debug!("Finished encoding response: [{:?}]", buf);
}
