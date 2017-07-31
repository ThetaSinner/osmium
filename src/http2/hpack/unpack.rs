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
use http2::hpack::string;
use http2::hpack::context;
use http2::hpack::table;
use http2::hpack::flags;

#[derive(Debug)]
pub struct UnpackedHeaders {
    pub headers: header::Headers,
    pub octets_read: usize
}

pub fn unpack(data: &[u8], context: &mut context::Context) -> UnpackedHeaders {
    let mut unpacked_headers = UnpackedHeaders {
        headers: header::Headers::new(),
        octets_read: 0
    };

    let mut data_iter = data.iter().peekable();

    // TODO termination condition? just the flags failing to match isn't enough to end the loop correctly surely.
    // though if the slice passed in contains only a header list to unpack then the peek will correctly end the loop.
    while let Some(&&peek_front) = data_iter.peek() {
        // TODO a 0 in the MSB with an indexed header representation is an error, check for that after checking for other representations?
        // unfortunately that would mean decoding the number and making sure there are no strings after it otherise that'd be another representation...
        if peek_front & flags::INDEXED_HEADER_FLAG == flags::INDEXED_HEADER_FLAG {
            let decoded_number = number::decode(&mut data_iter, 7);
            unpacked_headers.octets_read += decoded_number.octets_read;

            let field = context.get(decoded_number.num as usize).unwrap().clone();

            unpacked_headers.headers.push_header(header::Header::from(field));
        }
        else if peek_front & flags::LITERAL_WITH_INDEXING_FLAG == flags::LITERAL_WITH_INDEXING_FLAG {
            let decoded_number = number::decode(&mut data_iter, 6);
            unpacked_headers.octets_read += decoded_number.octets_read;

            // Note that headers are indexed from 1, so a zero value here means the name is not indexed.
            if decoded_number.num != 0 {
                let mut field = context.get(decoded_number.num as usize).unwrap().clone();

                let decoded_string = string::decode(&mut data_iter);
                unpacked_headers.octets_read += decoded_string.octets_read;
                // the header is indexed but we want to use the value from the packed header.
                field.value = decoded_string.string;

                // this representation causes the field to be added to the dynamic table.
                context.insert(field.clone());

                unpacked_headers.headers.push_header(header::Header::from(field));
            }
            else {
                let decoded_name = string::decode(&mut data_iter);
                unpacked_headers.octets_read += decoded_name.octets_read;

                let decoded_value = string::decode(&mut data_iter);
                unpacked_headers.octets_read += decoded_value.octets_read;

                let field = table::Field {
                    name: decoded_name.string,
                    value: decoded_value.string
                };

                // this representation causes the field to be added to the dynamic table.
                context.insert(field.clone());

                unpacked_headers.headers.push_header(header::Header::from(field));
            }
        }
        else if peek_front & flags::LITERAL_WITHOUT_INDEXING_FLAG == 0 {
            let decoded_number = number::decode(&mut data_iter, 4);
            unpacked_headers.octets_read += decoded_number.octets_read;

            // Note that headers are indexed from 1, so a zero value here means the name is not indexed.
            if decoded_number.num != 0 {
                let mut field = context.get(decoded_number.num as usize).unwrap().clone();

                let decoded_string = string::decode(&mut data_iter);
                unpacked_headers.octets_read += decoded_string.octets_read;
                // the header is indexed but we want to use the value from the packed header.
                field.value = decoded_string.string;

                unpacked_headers.headers.push_header(header::Header::from(field));
            }
            else {
                let decoded_name = string::decode(&mut data_iter);
                unpacked_headers.octets_read += decoded_name.octets_read;

                let decoded_value = string::decode(&mut data_iter);
                unpacked_headers.octets_read += decoded_value.octets_read;

                let field = table::Field {
                    name: decoded_name.string,
                    value: decoded_value.string
                };

                unpacked_headers.headers.push_header(header::Header::from(field));
            }
        }
        else if peek_front & flags::LITERAL_NEVER_INDEX_FLAG == flags::LITERAL_NEVER_INDEX_FLAG {
            // TODO the output header needs to be marked, because the server is responsible for propogating the never index flag.

            let decoded_number = number::decode(&mut data_iter, 4);
            unpacked_headers.octets_read += 2 + decoded_number.octets_read;

            // Note that headers are indexed from 1, so a zero value here means the name is not indexed.
            let mut header = if decoded_number.num != 0 {
                let mut field = context.get(decoded_number.num as usize).unwrap().clone();

                let decoded_string = string::decode(&mut data_iter);
                unpacked_headers.octets_read += decoded_string.octets_read;
                // the header is indexed but we want to use the value from the packed header.
                field.value = decoded_string.string;

                header::Header::from(field)
            }
            else {
                let decoded_name = string::decode(&mut data_iter);
                unpacked_headers.octets_read += decoded_name.octets_read;

                let decoded_value = string::decode(&mut data_iter);
                unpacked_headers.octets_read += decoded_value.octets_read;

                let field = table::Field {
                    name: decoded_name.string,
                    value: decoded_value.string
                };

                header::Header::from(field)
            };

            // the header field should never be indexed, primarily to intended to protect values which are not to
            // be put at risk by compressing them. Therefore, set allow compression flag to false.
            header.set_allow_compression(false);
            unpacked_headers.headers.push_header(header);
        }
        else if peek_front & flags::SIZE_UPDATE_FLAG == flags::SIZE_UPDATE_FLAG {
            let decoded_number = number::decode(&mut data_iter, 5);
            unpacked_headers.octets_read += decoded_number.octets_read;

            // TODO the new value must be below the maximum specified by the protocol using hpack, in this case http2
            // as this is being written first it will have to be modified once http2 settings can be decoded in the http2 module.

            context.set_max_size(decoded_number.num as usize);
        }
        else {
            break;
        }
    }

    unpacked_headers
}

impl From<table::Field> for header::Header {
    fn from(field: table::Field) -> Self {
        let header_name = header::HeaderName::from(field.name.as_ref());

        header::Header::new(
            header_name.clone(), 
            match header_name {
                // TODO map types which should be numbers etc.
                _ => header::HeaderValue::Str(field.value)
            }
        )
    }
}
