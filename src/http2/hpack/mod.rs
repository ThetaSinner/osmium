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

pub mod number;
pub mod string;
pub mod table;
pub mod context;
pub mod huffman;
pub mod unpack;
pub mod pack;
pub mod flags;

use self::table::{Table, Field};
use self::context::Context;

pub const STATIC_TABLE_LENGTH: usize = 61;

pub struct HPack {
    // TODO not sure this needs to be stored, and applied to every created table, it can just be hardcoded for now.
    // The maximum total storage size allowed for lookup tables
    //max_table_size_setting: usize,

    /// The single static table instance to be shared by all contexts provided by this `HPack` instance
    static_table: Table
}

impl HPack {
    pub fn new() -> Self {
        let mut static_table = Table::new();
        
        static_table.push_back(Field{name: String::from(":authority"), value: String::from("")});
        static_table.push_back(Field{name: String::from(":method"), value: String::from("GET")});
        static_table.push_back(Field{name: String::from(":method"), value: String::from("POST")});
        static_table.push_back(Field{name: String::from(":path"), value: String::from("/")});
        static_table.push_back(Field{name: String::from(":path"), value: String::from("/index.html")});
        static_table.push_back(Field{name: String::from(":scheme"), value: String::from("http")});
        static_table.push_back(Field{name: String::from(":scheme"), value: String::from("https")});
        static_table.push_back(Field{name: String::from(":status"), value: String::from("200")});
        static_table.push_back(Field{name: String::from(":status"), value: String::from("204")});
        static_table.push_back(Field{name: String::from(":status"), value: String::from("206")});
        static_table.push_back(Field{name: String::from(":status"), value: String::from("304")});
        static_table.push_back(Field{name: String::from(":status"), value: String::from("400")});
        static_table.push_back(Field{name: String::from(":status"), value: String::from("404")});
        static_table.push_back(Field{name: String::from(":status"), value: String::from("500")});
        static_table.push_back(Field{name: String::from("accept-charset"), value: String::from("")});
        static_table.push_back(Field{name: String::from("accept-encoding"), value: String::from("gzip, deflate")});
        static_table.push_back(Field{name: String::from("accept-language"), value: String::from("")});
        static_table.push_back(Field{name: String::from("accept-ranges"), value: String::from("")});
        static_table.push_back(Field{name: String::from("accept"), value: String::from("")});
        static_table.push_back(Field{name: String::from("access-control-allow-origin"), value: String::from("")});
        static_table.push_back(Field{name: String::from("age"), value: String::from("")});
        static_table.push_back(Field{name: String::from("allow"), value: String::from("")});
        static_table.push_back(Field{name: String::from("authorization"), value: String::from("")});
        static_table.push_back(Field{name: String::from("cache-control"), value: String::from("")});
        static_table.push_back(Field{name: String::from("content-disposition"), value: String::from("")});
        static_table.push_back(Field{name: String::from("content-encoding"), value: String::from("")});
        static_table.push_back(Field{name: String::from("content-language"), value: String::from("")});
        static_table.push_back(Field{name: String::from("content-length"), value: String::from("")});
        static_table.push_back(Field{name: String::from("content-location"), value: String::from("")});
        static_table.push_back(Field{name: String::from("content-range"), value: String::from("")});
        static_table.push_back(Field{name: String::from("content-type"), value: String::from("")});
        static_table.push_back(Field{name: String::from("cookie"), value: String::from("")});
        static_table.push_back(Field{name: String::from("date"), value: String::from("")});
        static_table.push_back(Field{name: String::from("etag"), value: String::from("")});
        static_table.push_back(Field{name: String::from("expect"), value: String::from("")});
        static_table.push_back(Field{name: String::from("expires"), value: String::from("")});
        static_table.push_back(Field{name: String::from("from"), value: String::from("")});
        static_table.push_back(Field{name: String::from("host"), value: String::from("")});
        static_table.push_back(Field{name: String::from("if-match"), value: String::from("")});
        static_table.push_back(Field{name: String::from("if-modified-since"), value: String::from("")});
        static_table.push_back(Field{name: String::from("if-none-match"), value: String::from("")});
        static_table.push_back(Field{name: String::from("if-range"), value: String::from("")});
        static_table.push_back(Field{name: String::from("if-unmodified-since"), value: String::from("")});
        static_table.push_back(Field{name: String::from("last-modified"), value: String::from("")});
        static_table.push_back(Field{name: String::from("link"), value: String::from("")});
        static_table.push_back(Field{name: String::from("location"), value: String::from("")});
        static_table.push_back(Field{name: String::from("max-forwards"), value: String::from("")});
        static_table.push_back(Field{name: String::from("proxy-authenticate"), value: String::from("")});
        static_table.push_back(Field{name: String::from("proxy-authorization"), value: String::from("")});
        static_table.push_back(Field{name: String::from("range"), value: String::from("")});
        static_table.push_back(Field{name: String::from("referer"), value: String::from("")});
        static_table.push_back(Field{name: String::from("refresh"), value: String::from("")});
        static_table.push_back(Field{name: String::from("retry-after"), value: String::from("")});
        static_table.push_back(Field{name: String::from("server"), value: String::from("")});
        static_table.push_back(Field{name: String::from("set-cookie"), value: String::from("")});
        static_table.push_back(Field{name: String::from("strict-transport-security"), value: String::from("")});
        static_table.push_back(Field{name: String::from("transfer-encoding"), value: String::from("")});
        static_table.push_back(Field{name: String::from("user-agent"), value: String::from("")});
        static_table.push_back(Field{name: String::from("vary"), value: String::from("")});
        static_table.push_back(Field{name: String::from("via"), value: String::from("")});
        static_table.push_back(Field{name: String::from("www-authenticate"), value: String::from("")});

        assert_eq!(STATIC_TABLE_LENGTH, static_table.len(), "static table should have 61 entries");

        HPack {
            static_table: static_table
        }
    }

    pub fn new_context(&self) -> Context {
        Context::new(&self.static_table)
    }
}

#[cfg(test)]
mod tests {
    use super::{HPack, pack, unpack, context};
    use http2::header;

    fn to_hex_dump(data: &[u8]) -> String {
        println!("{:?}, {}", data, data.len());

        let mut hex = Vec::new();
        for v in data {
            hex.push(if v < &16 {
                format!("0{:x}", v).to_owned()
            }
            else {
                format!("{:x}", v).to_owned()
            });
        }

        let mut dump = hex.chunks(2).map(|x| {
            if x.len()  == 2 {
                format!("{}{} ", x[0], x[1]).to_owned()
            }
            else {
                format!("{} ", x[0]).to_owned()
            }
        }).fold(String::from(""), |acc, el| {
            acc + &el
        });

        dump.pop();
        dump
    }

    fn assert_headers(expected: &header::Headers, actual: &header::Headers) {
        assert_eq!(expected.len(), actual.len());

        let mut actual_iter = actual.iter();
        for expected_header in expected.iter() {
            let actual_header = actual_iter.next().unwrap();
            assert_eq!(expected_header.name, actual_header.name);
            assert_eq!(expected_header.value, actual_header.value);
            assert_eq!(expected_header.is_allow_compression(), actual_header.is_allow_compression());
        }
    }

    fn assert_table_entry(context: &context::Context, index: usize, name: &str, value: &str) {
        let dynamic_table_entry = context.get(index);
        assert!(dynamic_table_entry.is_some());
        let field = dynamic_table_entry.unwrap();
        assert_eq!(name, &field.name);
        assert_eq!(value, &field.value);
    }

    // See C.2.1 encode
    #[test]
    pub fn encode_custom_header() {
        let hpack = HPack::new();
        let mut encoding_context = hpack.new_context();

        let mut headers = header::Headers::new();

        headers.push(
            header::HeaderName::CustomHeader(String::from("custom-key")),
            header::HeaderValue::Str(String::from("custom-header"))
        );

        let encoded = pack::pack(&headers, &mut encoding_context, false);

        assert_eq!("400a 6375 7374 6f6d 2d6b 6579 0d63 7573 746f 6d2d 6865 6164 6572", to_hex_dump(encoded.as_slice()));
    }

    // C.2.1 decode
    #[test]
    pub fn decode_custom_header() {
        let hpack = HPack::new();
        let mut encoding_context = hpack.new_context();

        let mut headers = header::Headers::new();

        headers.push(
            header::HeaderName::CustomHeader(String::from("custom-key")),
            header::HeaderValue::Str(String::from("custom-header"))
        );

        let encoded = pack::pack(&headers, &mut encoding_context, false);

        let mut decoding_context = hpack.new_context();
        let decoded = unpack::unpack(&encoded, &mut decoding_context);

        // assert the decoded headers.
        assert_headers(&headers, &decoded.headers);
        
        // assert the dynamic table.
        assert_eq!(55, decoding_context.size());
        assert_table_entry(&decoding_context, 62, "custom-key", "custom-header");
    }

    // See C.2.2 encode
    #[test]
    pub fn encode_literal_field_without_indexing() {
        let hpack = HPack::new();
        let mut encoding_context = hpack.new_context();

        let mut headers = header::Headers::new();

        headers.push(
            header::HeaderName::PseudoPath,
            header::HeaderValue::Str(String::from("/sample/path"))
        );

        let encoded = pack::pack(&headers, &mut encoding_context, false);

        assert_eq!("040c 2f73 616d 706c 652f 7061 7468", to_hex_dump(encoded.as_slice()));
    }

    // See C.2.2 decode
    #[test]
    pub fn decode_literal_field_without_indexing() {
        let hpack = HPack::new();
        let mut encoding_context = hpack.new_context();

        let mut headers = header::Headers::new();

        headers.push(
            header::HeaderName::PseudoPath,
            header::HeaderValue::Str(String::from("/sample/path"))
        );

        let encoded = pack::pack(&headers, &mut encoding_context, false);

        let mut decoding_context = hpack.new_context();
        let decoded = unpack::unpack(&encoded, &mut decoding_context);

        // assert the decoded headers.
        assert_headers(&headers, &decoded.headers);
        
        // assert the dynamic table.
        assert_eq!(0, decoding_context.size());
    }

    // TODO this uses the do not compress route to never indexed, add a test which
    // uses a preset never indexed header name.

    // See C.2.3 encode
    #[test]
    pub fn encode_literal_field_never_indexed() {
        let hpack = HPack::new();
        let mut encoding_context = hpack.new_context();

        let mut headers = header::Headers::new();

        let mut secret_password_header = header::Header::new(
            header::HeaderName::CustomHeader(String::from("password")),
            header::HeaderValue::Str(String::from("secret"))
        );
        secret_password_header.set_allow_compression(false);

        headers.push_header(secret_password_header);

        let encoded = pack::pack(&headers, &mut encoding_context, false);

        assert_eq!("1008 7061 7373 776f 7264 0673 6563 7265 74", to_hex_dump(encoded.as_slice()));
    }

    // See C.2.3 decode
    #[test]
    pub fn decode_literal_field_never_indexed() {
        let hpack = HPack::new();
        let mut encoding_context = hpack.new_context();

        let mut headers = header::Headers::new();

        let mut secret_password_header = header::Header::new(
            header::HeaderName::CustomHeader(String::from("password")),
            header::HeaderValue::Str(String::from("secret"))
        );
        secret_password_header.set_allow_compression(false);

        headers.push_header(secret_password_header);

        let encoded = pack::pack(&headers, &mut encoding_context, false);

        let mut decoding_context = hpack.new_context();
        let decoded = unpack::unpack(&encoded, &mut decoding_context);

        // assert the decoded headers.
        assert_headers(&headers, &decoded.headers);
        
        // assert the dynamic table.
        assert_eq!(0, decoding_context.size());
    }

    // See C.2.4 encode
    #[test]
    pub fn encode_indexed() {
        let hpack = HPack::new();
        let mut encoding_context = hpack.new_context();

        let mut headers = header::Headers::new();

        let method_get_header = header::Header::new(
            header::HeaderName::PseudoMethod,
            header::HeaderValue::Str(String::from("GET"))
        );

        headers.push_header(method_get_header);

        let encoded = pack::pack(&headers, &mut encoding_context, false);

        assert_eq!("82", to_hex_dump(encoded.as_slice()));
    }

    // See C.2.4 decode
    #[test]
    pub fn decode_indexed() {
        let hpack = HPack::new();
        let mut encoding_context = hpack.new_context();

        let mut headers = header::Headers::new();

        let method_get_header = header::Header::new(
            header::HeaderName::PseudoMethod,
            header::HeaderValue::Str(String::from("GET"))
        );

        headers.push_header(method_get_header);

        let encoded = pack::pack(&headers, &mut encoding_context, false);

        let mut decoding_context = hpack.new_context();
        let decoded = unpack::unpack(&encoded, &mut decoding_context);

        // assert the decoded headers.
        assert_headers(&headers, &decoded.headers);
        
        // assert the dynamic table.
        assert_eq!(0, decoding_context.size());
    }

    // TODO the tests above should assert the encoding context as well as the decoding context
    // to ensure they match.

    fn section_c_three_and_four_requests(
        use_huffman_coding: bool,
        request_one_hex_dump: &str,
        request_two_hex_dump: &str,
        request_three_hex_dump: &str
    ) {
        let hpack = HPack::new();

        // These two contexts are the only state, everything else is only used in request processing.
        let mut encoding_context = hpack.new_context();
        let mut decoding_context = hpack.new_context();

        // Request 1
        {
            let mut headers = header::Headers::new();
            headers.push(
                header::HeaderName::PseudoMethod,
                header::HeaderValue::Str(String::from("GET"))
            );
            headers.push(
                header::HeaderName::PseudoScheme,
                header::HeaderValue::Str(String::from("http"))
            );
            headers.push(
                header::HeaderName::PseudoPath,
                header::HeaderValue::Str(String::from("/"))
            );
            headers.push(
                header::HeaderName::PseudoAuthority,
                header::HeaderValue::Str(String::from("www.example.com"))
            );

            let encoded = pack::pack(&headers, &mut encoding_context, use_huffman_coding);
            assert_eq!(request_one_hex_dump, to_hex_dump(encoded.as_slice()));

            let decoded = unpack::unpack(&encoded, &mut decoding_context);

            // assert the decoded headers.
            assert_headers(&headers, &decoded.headers);
            
            // assert the dynamic table.
            assert_eq!(57, decoding_context.size());
            assert_table_entry(&decoding_context, 62, ":authority", "www.example.com");

            // assert that the encoding context is the same as the decoding context
            // TODO extract function.
            assert_eq!(decoding_context.size(), encoding_context.size());
            assert_table_entry(&encoding_context, 62, ":authority", "www.example.com");
        }

        // Request 2
        {
            let mut headers = header::Headers::new();
            headers.push(
                header::HeaderName::PseudoMethod,
                header::HeaderValue::Str(String::from("GET"))
            );
            headers.push(
                header::HeaderName::PseudoScheme,
                header::HeaderValue::Str(String::from("http"))
            );
            headers.push(
                header::HeaderName::PseudoPath,
                header::HeaderValue::Str(String::from("/"))
            );
            headers.push(
                header::HeaderName::PseudoAuthority,
                header::HeaderValue::Str(String::from("www.example.com"))
            );
            headers.push(
                header::HeaderName::CacheControl,
                header::HeaderValue::Str(String::from("no-cache"))
            );

            let encoded = pack::pack(&headers, &mut encoding_context, use_huffman_coding);
            assert_eq!(request_two_hex_dump, to_hex_dump(encoded.as_slice()));

            let decoded = unpack::unpack(&encoded, &mut decoding_context);

            // assert the decoded headers.
            assert_headers(&headers, &decoded.headers);
            
            // assert the dynamic table.
            assert_eq!(110, decoding_context.size());
            assert_table_entry(&decoding_context, 62, "cache-control", "no-cache");
            assert_table_entry(&decoding_context, 63, ":authority", "www.example.com");

            // assert that the encoding context is the same as the decoding context
            // TODO extract function.
            assert_eq!(decoding_context.size(), encoding_context.size());
            assert_table_entry(&encoding_context, 62, "Cache-Control", "no-cache");
            assert_table_entry(&encoding_context, 63, ":authority", "www.example.com");
        }

        // Third request
        {
            let mut headers = header::Headers::new();
            headers.push(
                header::HeaderName::PseudoMethod,
                header::HeaderValue::Str(String::from("GET"))
            );
            headers.push(
                header::HeaderName::PseudoScheme,
                header::HeaderValue::Str(String::from("https"))
            );
            headers.push(
                header::HeaderName::PseudoPath,
                header::HeaderValue::Str(String::from("/index.html"))
            );
            headers.push(
                header::HeaderName::PseudoAuthority,
                header::HeaderValue::Str(String::from("www.example.com"))
            );
            headers.push(
                header::HeaderName::CustomHeader(String::from("custom-key")),
                header::HeaderValue::Str(String::from("custom-value"))
            );

            let encoded = pack::pack(&headers, &mut encoding_context, use_huffman_coding);
            assert_eq!(request_three_hex_dump, to_hex_dump(encoded.as_slice()));

            let decoded = unpack::unpack(&encoded, &mut decoding_context);

            // assert the decoded headers.
            assert_headers(&headers, &decoded.headers);
            
            // assert the dynamic table.
            assert_eq!(164, decoding_context.size());
            assert_table_entry(&decoding_context, 62, "custom-key", "custom-value");
            assert_table_entry(&decoding_context, 63, "cache-control", "no-cache");
            assert_table_entry(&decoding_context, 64, ":authority", "www.example.com");

            // assert that the encoding context is the same as the decoding context
            // TODO extract function.
            assert_eq!(decoding_context.size(), encoding_context.size());
            assert_table_entry(&encoding_context, 62, "custom-key", "custom-value");
            assert_table_entry(&encoding_context, 63, "Cache-Control", "no-cache");
            assert_table_entry(&encoding_context, 64, ":authority", "www.example.com");
        }
    }

    use pretty_env_logger;

    // See C.3 process multiple requests on the same context
    #[test]
    pub fn decode_multiple_requests_without_huffman_coding() {
        section_c_three_and_four_requests(
            false,
            "8286 8441 0f77 7777 2e65 7861 6d70 6c65 2e63 6f6d",
            "8286 84be 5808 6e6f 2d63 6163 6865",
            "8287 85bf 400a 6375 7374 6f6d 2d6b 6579 0c63 7573 746f 6d2d 7661 6c75 65"
        );
    }

    // See C.3 process multiple requests on the same context with huffman coding
    #[test]
    pub fn decode_multiple_requests_with_huffman_coding() {
        pretty_env_logger::init().unwrap();

        section_c_three_and_four_requests(
            true,
            "8286 8441 8cf1 e3c2 e5f2 3a6b a0ab 90f4 ff",
            "8286 84be 5886 a8eb 1064 9cbf",
            "8287 85bf 4088 25a8 49e9 5ba9 7d7f 8925 a849 e95b b8e8 b4bf"
        );
    }
}
