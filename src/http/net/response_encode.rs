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

/// Convert a `Response` struct to http transport format
///
/// This is called from the http codec which is the tokio tranport format conversion.
pub fn encode(res: Response, buf: &mut BytesMut) {
    write!(buf, "HTTP/{} OK\r\nServer: Osmium\r\nContent-Length: 0\r\n\r\n", res.version).unwrap();
}
