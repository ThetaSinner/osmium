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
use std::slice;

// osmium
use http2::hpack::context::{self, ContextTrait};
use http2::hpack::flags;
use http2::hpack::number;
use http2::hpack::string;
use http2::hpack::table;
use http2::hpack::header_trait;

const NEVER_INDEXED: [String; 0] = [
    // Date is a temporary example of a never indexed header
    //header::HeaderName::Date
];

const LITERAL_WITHOUT_INDEXING: [&str; 1] = [
    // Path is an example of a header which has common values which should be indexed i.e. '/' and '/index.html'
    // but which will have many values which should not be fully indexed, just header name indexed
    ":path"
];

// TODO the table size update needs to come through here I think?
// how would that work.. hopefully there'll be an example in the spec

// TODO per header huffman coding setting?

// TODO comments need updating, they're still the ones I wrote while puzzling out the encoder.

pub fn pack<T>(headers: slice::Iter<T>, context: &mut context::SendContext, use_huffman_coding: bool) -> Vec<u8>
    where T: header_trait::HpackHeaderTrait
{
    let mut target = Vec::new();

    // Check whether a decision has been made to change the dynamic table size.
    if let Some(size_update) = context.get_size_update() {
        // Update the size of the dynamic table used by the send context. This may cause evictions
        // if the size is reduced.
        // This could be done as soon as the decision to change the size is made, which might free
        // up memory sooner. However, doing it here means that the change is always made at the same 
        // time as the signal to the remote table is created.
        // TODO handle error here if the size_update is larger than the allowed size?
        // TODO why is that taking usize?
        context.set_max_size(size_update as usize);

        // The size update signal is sent to the remote decoding table.
        pack_dynamic_table_size_update(size_update, &mut target);
    }

    for header in headers {
        let field = table::Field {
            name: String::from(header.get_name()),
            value: String::from(header.get_value())
        };

        trace!("{:?}", field);

        if !header.is_allow_compression() || is_never_index_header(&field.name) {
            // TODO it's really not clever to have to clone the value here to build a field for search.
            // especially as find_field is never used without a Header available.
            if let Some((index, _)) = context.find_field(&field) {
                pack_literal_never_indexed_with_indexed_name(index, &field.value, &mut target);
            }
            else {
                pack_literal_never_indexed(&field, use_huffman_coding, &mut target);
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
                    if is_literal_without_indexing_header(&field.name) {
                        trace!("pack without indexing");
                        pack_literal_without_indexing_with_indexed_name(index, &field.value, use_huffman_coding, &mut target);
                    }
                    else {
                        pack_literal_with_indexing_with_indexed_name(index, &field.value, use_huffman_coding, &mut target);
                        context.insert(field);
                    }
                }
            }
            else {
                trace!("not found, start from scratch");

                // header name is not currently indexed, we can index it now, or send a literal representation.
                if is_literal_without_indexing_header(&field.name) {
                    pack_literal_without_indexing(&field, use_huffman_coding, &mut target);
                }
                else {
                    pack_literal_with_indexing(&field, use_huffman_coding, &mut target);
                    context.insert(field);
                }
            }
        }
    }

    target
}

fn is_never_index_header(header_name: &str) -> bool {
    for never_index_header_name in NEVER_INDEXED.into_iter() {
        if header_name == never_index_header_name {
            return true;
        }
    }

    false
}

fn is_literal_without_indexing_header(header_name: &String) -> bool {
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

fn pack_literal_with_indexing_with_indexed_name(index: usize, header_value: &str, use_huffman_coding: bool, target: &mut Vec<u8>) {
    let encoded_name_index = number::encode(index as u32, 6);

    target.push(flags::LITERAL_WITH_INDEXING_FLAG | encoded_name_index.prefix);
    if let Some(rest) = encoded_name_index.rest {
        target.extend(rest);
    }

    trace!("{:?}", string::encode(String::from(header_value), use_huffman_coding));

    target.extend(string::encode(String::from(header_value), use_huffman_coding));
}

fn pack_literal_with_indexing(field: &table::Field, use_huffman_coding: bool, target: &mut Vec<u8>) {
    target.push(flags::LITERAL_WITH_INDEXING_FLAG);

    target.extend(string::encode(String::from(field.name.clone()), use_huffman_coding));
    target.extend(string::encode(String::from(field.value.clone()), use_huffman_coding));
}

fn pack_literal_without_indexing_with_indexed_name(index: usize, header_value: &str, use_huffman_coding: bool, target: &mut Vec<u8>) {
    trace!("index to use {}", index);
    let encoded_name_index = number::encode(index as u32, 4);

    trace!("prefix {}", encoded_name_index.prefix);

    target.push((!flags::LITERAL_WITH_INDEXING_FLAG) & encoded_name_index.prefix);
    if let Some(rest) = encoded_name_index.rest {
        target.extend(rest);
    }

    target.extend(string::encode(String::from(header_value.clone()), use_huffman_coding));
}

fn pack_literal_without_indexing(field: &table::Field, use_huffman_coding: bool, target: &mut Vec<u8>) {
    target.push(0u8);

    target.extend(string::encode(String::from(field.name.clone()), use_huffman_coding));
    target.extend(string::encode(String::from(field.value.clone()), use_huffman_coding));
}

fn pack_literal_never_indexed_with_indexed_name(index: usize, header_value: &str, target: &mut Vec<u8>) {
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

fn pack_literal_never_indexed(field: &table::Field, use_huffman_coding: bool, target: &mut Vec<u8>) {
    target.push(flags::LITERAL_NEVER_INDEX_FLAG);

    target.extend(string::encode(String::from(field.name.clone()), use_huffman_coding));
    // deliberately do not allow override of huffman coding for the value
    target.extend(string::encode(String::from(field.value.clone()), false));
}

fn pack_dynamic_table_size_update(size_update: u32, target: &mut Vec<u8>) {
    let encoded_size_update = number::encode(size_update, 5);

    target.push(flags::SIZE_UPDATE_FLAG | encoded_size_update.prefix);
    if let Some(rest) = encoded_size_update.rest {
        target.extend(rest);
    }
}
