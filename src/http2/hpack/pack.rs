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
use http2::hpack::context;
use http2::hpack::flags;
use http2::hpack::number;
use http2::hpack::string;
use http2::hpack::table;

const NEVER_INDEXED: [header::HeaderName; 1] = [
    header::HeaderName::Date
];

pub fn pack(headers: &header::Headers, context: &mut context::Context) -> Vec<u8> {
    let mut target = Vec::new();

    for header in headers.iter() {
        let field = table::Field {
            name: String::from(header.name.clone()),
            value: String::from(header.value.clone())
        };

        if !header.is_allow_compression() || is_never_index_header(&header.name) {
            // TODO it's really not clever to have to clone the value here to build a field for search.
            // especially as find_field is never used without a Header available.
            if let Some((index, _)) = context.find_field(&field) {
                pack_literal_never_indexed_with_indexed_name(index, &header.value, &mut target);
            }
            else {
                pack_literal_never_indexed(&header, &mut target);
            }
        }
        else {
            if let Some((index, with_value)) = context.find_field(&field) {
                // header name is indexed and value is indexed as indicated by with_value.
                if with_value {
                    pack_indexed_header(index, &mut target);
                }
                else {
                    // the value is not currently indexed, we could index and allow the value to be added to the 
                    // dynamic table in the decoder, or we could not index and just refer to this header name.
                    pack_literal_with_indexing_with_indexed_name(index, &header.value, &mut target);
                    context.insert(field);
                }
            }
            else {
                // header name is not currently indexed, we can index it now, or send a literal representation.
                pack_literal_with_indexing(&header, &mut target);
                context.insert(field);
            }
        }
    }

    target
}

fn is_never_index_header(header_name: &header::HeaderName) -> bool {
    for never_index_header_name in NEVER_INDEXED.into_iter() {
        if header_name == never_index_header_name {
            return true;
        }
    }

    false
}

fn pack_indexed_header(index: usize, target: &mut Vec<u8>) {
    let encoded_index = number::encode(index as u32, 7);

    target.push(flags::INDEXED_HEADER_FLAG | encoded_index.prefix);
    if let Some(rest) = encoded_index.rest {
        target.extend(rest);
    }
}

fn pack_literal_with_indexing_with_indexed_name(index: usize, header_value: &header::HeaderValue, target: &mut Vec<u8>) {
    let encoded_name_index = number::encode(index as u32, 6);

    target.push(flags::LITERAL_WITH_INDEXING_FLAG | encoded_name_index.prefix);
    if let Some(rest) = encoded_name_index.rest {
        target.extend(rest);
    }

    // TODO there is no way to disable huffman coding (if there were a reason to do so)
    target.extend(string::encode(String::from(header_value.clone()), true));
}

fn pack_literal_with_indexing(header: &header::Header, target: &mut Vec<u8>) {
    target.push(flags::LITERAL_WITH_INDEXING_FLAG);

    target.extend(string::encode(String::from(header.name.clone()), true));
    target.extend(string::encode(String::from(header.value.clone()), true));
}

fn pack_literal_never_indexed_with_indexed_name(index: usize, header_value: &header::HeaderValue, target: &mut Vec<u8>) {
    let encoded_name_index = number::encode(index as u32, 4);

    target.push(flags::LITERAL_NEVER_INDEX_FLAG | encoded_name_index.prefix);
    if let Some(rest) = encoded_name_index.rest {
        target.extend(rest);
    }

    // field should not be compressed... which means not indexed but the spec is not clear
    // what should be done with regards to huffman coding.
    target.extend(string::encode(String::from(header_value.clone()), false));
}

fn pack_literal_never_indexed(header: &header::Header, target: &mut Vec<u8>) {
    target.push(flags::LITERAL_NEVER_INDEX_FLAG);

    target.extend(string::encode(String::from(header.name.clone()), true));
    target.extend(string::encode(String::from(header.value.clone()), false));
}
