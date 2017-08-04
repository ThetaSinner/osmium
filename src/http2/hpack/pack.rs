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
    // Date is a temporary example of a never indexed header
    header::HeaderName::Date
];

const LITERAL_WITHOUT_INDEXING: [header::HeaderName; 1] = [
    // Path is an example of a header which has common values which should be indexed i.e. '/' and '/index.html'
    // but which will have many values which should not be fully indexed, just header name indexed
    header::HeaderName::PseudoPath
];

// TODO the table size update needs to come through here I think?
// how would that work.. hopefully there'll be an example in the spec

// TODO per header huffman coding setting?

// TODO comments need updating, they're still the ones I wrote while puzzling out the encoder.

pub fn pack(headers: &header::Headers, context: &mut context::Context, use_huffman_coding: bool) -> Vec<u8> {
    let mut target = Vec::new();

    for header in headers.iter() {
        let field = table::Field {
            name: String::from(header.name.clone()),
            value: String::from(header.value.clone())
        };

        trace!("{:?}", field);

        if !header.is_allow_compression() || is_never_index_header(&header.name) {
            // TODO it's really not clever to have to clone the value here to build a field for search.
            // especially as find_field is never used without a Header available.
            if let Some((index, _)) = context.find_field(&field) {
                pack_literal_never_indexed_with_indexed_name(index, &header.value, &mut target);
            }
            else {
                pack_literal_never_indexed(&header, use_huffman_coding, &mut target);
            }
        }
        else {
            trace!("okay, so we need to work out how to index this one");

            if let Some((index, with_value)) = context.find_field(&field) {
                trace!("found a field, with index {} and with value present {}", index, with_value);
                // header name is indexed and value is indexed as indicated by with_value.
                if with_value {
                    pack_indexed_header(index, &mut target);
                }
                else {
                    trace!("is indexed, but not with value");
                    // the value is not currently indexed, we could index and allow the value to be added to the 
                    // dynamic table in the decoder, or we could not index and just refer to this header name.
                    if is_literal_without_indexing_header(&header.name) {
                        trace!("pack without indexing");
                        pack_literal_without_indexing_with_indexed_name(index, &header.value, use_huffman_coding, &mut target);
                    }
                    else {
                        pack_literal_with_indexing_with_indexed_name(index, &header.value, use_huffman_coding, &mut target);
                        context.insert(field);
                    }
                }
            }
            else {
                trace!("not found, start from scratch");

                // header name is not currently indexed, we can index it now, or send a literal representation.
                if is_literal_without_indexing_header(&header.name) {
                    pack_literal_without_indexing(&header, use_huffman_coding, &mut target);
                }
                else {
                    pack_literal_with_indexing(&header, use_huffman_coding, &mut target);
                    context.insert(field);
                }
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

fn is_literal_without_indexing_header(header_name: &header::HeaderName) -> bool {
    for literal_without_indexing_header_name in LITERAL_WITHOUT_INDEXING.into_iter() {
        if header_name == literal_without_indexing_header_name {
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

    trace!("packed indexed header with index {}, {:?}", index, target);
}

fn pack_literal_with_indexing_with_indexed_name(index: usize, header_value: &header::HeaderValue, use_huffman_coding: bool, target: &mut Vec<u8>) {
    let encoded_name_index = number::encode(index as u32, 6);

    target.push(flags::LITERAL_WITH_INDEXING_FLAG | encoded_name_index.prefix);
    if let Some(rest) = encoded_name_index.rest {
        target.extend(rest);
    }

    target.extend(string::encode(String::from(header_value.clone()), use_huffman_coding));
}

fn pack_literal_with_indexing(header: &header::Header, use_huffman_coding: bool, target: &mut Vec<u8>) {
    target.push(flags::LITERAL_WITH_INDEXING_FLAG);

    target.extend(string::encode(String::from(header.name.clone()), use_huffman_coding));
    target.extend(string::encode(String::from(header.value.clone()), use_huffman_coding));
}

fn pack_literal_without_indexing_with_indexed_name(index: usize, header_value: &header::HeaderValue, use_huffman_coding: bool, target: &mut Vec<u8>) {
    trace!("index to use {}", index);
    let encoded_name_index = number::encode(index as u32, 4);

    target.push((!flags::LITERAL_WITH_INDEXING_FLAG) & encoded_name_index.prefix);
    if let Some(rest) = encoded_name_index.rest {
        target.extend(rest);
    }

    target.extend(string::encode(String::from(header_value.clone()), use_huffman_coding));
}

fn pack_literal_without_indexing(header: &header::Header, use_huffman_coding: bool, target: &mut Vec<u8>) {
    target.push(0u8);

    target.extend(string::encode(String::from(header.name.clone()), use_huffman_coding));
    target.extend(string::encode(String::from(header.value.clone()), use_huffman_coding));
}

fn pack_literal_never_indexed_with_indexed_name(index: usize, header_value: &header::HeaderValue, target: &mut Vec<u8>) {
    let encoded_name_index = number::encode(index as u32, 4);

    target.push(flags::LITERAL_NEVER_INDEX_FLAG | encoded_name_index.prefix);
    if let Some(rest) = encoded_name_index.rest {
        target.extend(rest);
    }

    // field should not be compressed... which means not indexed but the spec is not clear
    // what should be done with regards to huffman coding.
    // deliberately do not allow override of huffman coding for the value
    target.extend(string::encode(String::from(header_value.clone()), false));
}

fn pack_literal_never_indexed(header: &header::Header, use_huffman_coding: bool, target: &mut Vec<u8>) {
    target.push(flags::LITERAL_NEVER_INDEX_FLAG);

    target.extend(string::encode(String::from(header.name.clone()), use_huffman_coding));
    // deliberately do not allow override of huffman coding for the value
    target.extend(string::encode(String::from(header.value.clone()), false));
}
