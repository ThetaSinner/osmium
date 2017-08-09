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

use http2::hpack;
use http2::header;

pub fn use_hpack_so_the_unused_warnings_go_away() -> hpack::unpack::UnpackedHeaders {
    let header_pack = hpack::HPack::new();

    let mut headers = header::Headers::new();
    headers.push(header::HeaderName::Accept, header::HeaderValue::Str(String::from("text/plain")));

    let mut encoder_context = header_pack.new_context();
    let mut decoder_context = header_pack.new_context();

    let packed = hpack::pack::pack(&headers, &mut encoder_context, true);
    println!("{:?}", packed);
    hpack::unpack::unpack(packed.as_slice(), &mut decoder_context)
}

#[cfg(test)]
mod tests {
    use super::{use_hpack_so_the_unused_warnings_go_away};

    // TODO #[test]
    pub fn pack_unpack() {
        let unpacked = use_hpack_so_the_unused_warnings_go_away();

        for header in unpacked.headers.iter() {
            println!("{:?}", header);
        }
    }
}
