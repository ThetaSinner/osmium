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

// osmium
use http2::header;
use http2::hpack::number;
use http2::hpack::context;
use http2::hpack::table;

static INDEXED_HEADER_FLAG: u8 = 0x100;

pub struct UnpackedHeaders {
    pub headers: header::Headers,
    pub octets_read: u32
}

fn unpack(data: &[u8], context: &mut context::Context) -> UnpackedHeaders {
    let mut unpacked_headers = UnpackedHeaders {
        headers: header::Headers::new(),
        octets_read: 0
    };

    if data[0] & INDEXED_HEADER_FLAG {
        let decoded_number = number::decode(data, 7);

        let field = context.get(decoded_number.num).unwrap();
        
        unpacked_headers.headers.add_header(header::Header::from(field));
        unpacked_headers.octets_read += 1 + decoded_number.bits_read;
    }
}
