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
pub mod header_trait;

use self::table::{Table, Field};
use self::context::{SendContext, RecvContext, ContextTrait};

/// The number of items in the static table as defined in hpack section 2.3.1
pub const STATIC_TABLE_LENGTH: usize = 61;

/// Provides a container for the single instance of the static table,
/// and a method for constructing `Context` structures which have
/// a read-only reference to the static table instance.
pub struct HPack {
    /// The single static table instance to be shared by all contexts 
    /// provided by this `HPack` instance.
    static_table: Table
}

impl HPack {
    /// Creates a new `HPack` instance. 
    /// This method builds an instance of the predefined static table to be
    /// provided to send/receive contexts created from this instance.
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

    /// Get a new `SendContext` instance.
    pub fn new_send_context(&self) -> SendContext {
        SendContext::new(&self.static_table)
    }

    /// Get a new `RecvContext` instance.
    pub fn new_recv_context(&self) -> RecvContext {
        RecvContext::new(&self.static_table)
    }
}

#[cfg(test)]
mod tests {
    use super::{HPack, pack, unpack, table, STATIC_TABLE_LENGTH};
    use super::header_trait::HpackHeaderTrait;
    use http2::hpack::context::ContextTrait;
    
    // don't want a dependency on http2, so create fake http2 header implementation here
    #[derive(Debug)]
    struct FakeHeader {
        pub name: String,
        pub value: String,
        pub allow_compression: bool
    }

    impl HpackHeaderTrait for FakeHeader {
        fn from_field_with_compression_flag(field: table::Field, allow_compression: bool) -> Self {
            FakeHeader {
                name: field.name,
                value: field.value,
                allow_compression: allow_compression
            }
        }

        fn from_field(field: table::Field) -> Self {
            FakeHeader {
                name: field.name,
                value: field.value,
                allow_compression: true
            }
        }

        fn get_name(&self) -> String {
            self.name.clone()
        }

        fn get_value(&self) -> String {
            self.value.clone()
        }

        fn is_allow_compression(&self) -> bool {
            self.allow_compression
        }
    }

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

    fn assert_headers(expected: &Vec<FakeHeader>, actual: &Vec<FakeHeader>) {
        assert_eq!(expected.len(), actual.len());

        let mut actual_iter = actual.iter();
        for expected_header in expected.iter() {
            let actual_header = actual_iter.next().unwrap();
            assert_eq!(expected_header.name, actual_header.name);
            assert_eq!(expected_header.value, actual_header.value);
            assert_eq!(expected_header.is_allow_compression(), actual_header.is_allow_compression());
        }
    }

    // Note that the field name is converted to lower case. See http2::header for more.
    fn assert_table_entry<'a, T: ContextTrait<'a>>(context: &T, index: usize, name: &str, value: &str) {
        let dynamic_table_entry = context.get(index);
        assert!(dynamic_table_entry.is_some());
        let field = dynamic_table_entry.unwrap();
        assert_eq!(name, &field.name.to_lowercase());
        assert_eq!(value, &field.value);
    }

    fn assert_contexts_equal<'a, 'b, T: ContextTrait<'a>, S: ContextTrait<'b>>(expected_context: &T, actual_context: &S) {
        let mut index = STATIC_TABLE_LENGTH;

        // Checking the sizes first, as it makes it more likely the method will exit on an assert rather
        // than an index out of bounds below.
        assert_eq!(expected_context.size(), actual_context.size());

        loop {
            let entry = expected_context.get(index);

            if let Some(ref entry) = entry {
                assert_table_entry(actual_context, index, entry.name.to_lowercase().as_ref(), entry.value.as_ref());
                index += 1;
            }
            else {
                break;
            }
        }
    }

    // See C.2.1 encode
    #[test]
    pub fn encode_custom_header() {
        let hpack = HPack::new();
        let mut encoding_context = hpack.new_send_context();

        let mut headers = Vec::<FakeHeader>::new();

        headers.push(FakeHeader::from_field(table::Field {
            name: "custom-key".to_owned(),
            value: "custom-header".to_owned()
        }));

        let encoded = pack::pack(headers.iter(), &mut encoding_context, false);

        assert_eq!("400a 6375 7374 6f6d 2d6b 6579 0d63 7573 746f 6d2d 6865 6164 6572", to_hex_dump(encoded.as_slice()));
    }

    // C.2.1 decode
    #[test]
    pub fn decode_custom_header() {
        let hpack = HPack::new();

        let mut headers = Vec::<FakeHeader>::new();

        headers.push(FakeHeader::from_field(table::Field {
            name: "custom-key".to_owned(),
            value: "custom-header".to_owned()
        }));

        let mut encoding_context = hpack.new_send_context();
        let encoded = pack::pack(headers.iter(), &mut encoding_context, false);

        let mut decoding_context = hpack.new_recv_context();
        let mut decoded = unpack::UnpackedHeaders::<FakeHeader>::new();
        unpack::unpack(&encoded, &mut decoding_context, &mut decoded);

        // assert the decoded headers.
        assert_headers(&headers, &decoded.headers);
        
        // assert the dynamic table.
        assert_eq!(55, decoding_context.size());
        assert_table_entry(&decoding_context, 62, "custom-key", "custom-header");

        assert_contexts_equal(&decoding_context, &encoding_context);
    }

    // See C.2.2 encode
    #[test]
    pub fn encode_literal_field_without_indexing() {
        let hpack = HPack::new();
        let mut encoding_context = hpack.new_send_context();

        let mut headers = Vec::<FakeHeader>::new();

        headers.push(FakeHeader::from_field(table::Field {
            name: ":path".to_owned(),
            value: "/sample/path".to_owned()
        }));

        let encoded = pack::pack(headers.iter(), &mut encoding_context, false);

        assert_eq!("040c 2f73 616d 706c 652f 7061 7468", to_hex_dump(encoded.as_slice()));
    }

    // See C.2.2 decode
    #[test]
    pub fn decode_literal_field_without_indexing() {
        let hpack = HPack::new();

        let mut headers = Vec::<FakeHeader>::new();
        headers.push(FakeHeader::from_field(table::Field {
            name: ":path".to_owned(),
            value: "/sample/path".to_owned()
        }));

        let mut encoding_context = hpack.new_send_context();
        let encoded = pack::pack(headers.iter(), &mut encoding_context, false);

        let mut decoding_context = hpack.new_recv_context();
        let mut decoded = unpack::UnpackedHeaders::<FakeHeader>::new();
        unpack::unpack(&encoded, &mut decoding_context, &mut decoded);

        // assert the decoded headers.
        assert_headers(&headers, &decoded.headers);
        
        // assert the dynamic table.
        assert_eq!(0, decoding_context.size());

        assert_contexts_equal(&decoding_context, &encoding_context);
    }

    // TODO this uses the do not compress route to never indexed, add a test which
    // uses a preset never indexed header name.

    // See C.2.3 encode
    #[test]
    pub fn encode_literal_field_never_indexed() {
        let hpack = HPack::new();
        let mut encoding_context = hpack.new_send_context();

        let mut headers = Vec::<FakeHeader>::new();
        headers.push(FakeHeader::from_field_with_compression_flag(
            table::Field {
                name: "password".to_owned(),
                value: "secret".to_owned()
            }, 
            false
        ));

        let encoded = pack::pack(headers.iter(), &mut encoding_context, false);

        assert_eq!("1008 7061 7373 776f 7264 0673 6563 7265 74", to_hex_dump(encoded.as_slice()));
    }

    // See C.2.3 decode
    #[test]
    pub fn decode_literal_field_never_indexed() {
        let hpack = HPack::new();

        let mut headers = Vec::<FakeHeader>::new();
        headers.push(FakeHeader::from_field_with_compression_flag(
            table::Field {
                name: "password".to_owned(),
                value: "secret".to_owned()
            }, 
            false
        ));

        let mut encoding_context = hpack.new_send_context();
        let encoded = pack::pack(headers.iter(), &mut encoding_context, false);

        let mut decoding_context = hpack.new_recv_context();
        let mut decoded = unpack::UnpackedHeaders::<FakeHeader>::new();
        unpack::unpack(&encoded, &mut decoding_context, &mut decoded);

        // assert the decoded headers.
        assert_headers(&headers, &decoded.headers);
        
        // assert the dynamic table.
        assert_eq!(0, decoding_context.size());

        assert_contexts_equal(&decoding_context, &encoding_context);
    }

    // See C.2.4 encode
    #[test]
    pub fn encode_indexed() {
        let hpack = HPack::new();
        let mut encoding_context = hpack.new_send_context();

        let mut headers = Vec::<FakeHeader>::new();
        headers.push(FakeHeader::from_field(table::Field {
            name: ":method".to_owned(),
            value: "GET".to_owned()
        }));

        let encoded = pack::pack(headers.iter(), &mut encoding_context, false);

        assert_eq!("82", to_hex_dump(encoded.as_slice()));
    }

    // See C.2.4 decode
    #[test]
    pub fn decode_indexed() {
        let hpack = HPack::new();

        let mut headers = Vec::<FakeHeader>::new();
        headers.push(FakeHeader::from_field(table::Field {
            name: ":method".to_owned(),
            value: "GET".to_owned()
        }));

        let mut encoding_context = hpack.new_send_context();
        let encoded = pack::pack(headers.iter(), &mut encoding_context, false);

        let mut decoding_context = hpack.new_recv_context();
        let mut decoded = unpack::UnpackedHeaders::<FakeHeader>::new();
        unpack::unpack(&encoded, &mut decoding_context, &mut decoded);

        // assert the decoded headers.
        assert_headers(&headers, &decoded.headers);
        
        // assert the dynamic table.
        assert_eq!(0, decoding_context.size());

        assert_contexts_equal(&decoding_context, &encoding_context);
    }

    fn section_c_three_and_four_requests(
        use_huffman_coding: bool,
        request_one_hex_dump: &str,
        request_two_hex_dump: &str,
        request_three_hex_dump: &str
    ) {
        let hpack = HPack::new();

        // These two contexts are the only state, everything else is only used in request processing.
        let mut encoding_context = hpack.new_send_context();
        let mut decoding_context = hpack.new_recv_context();

        // Request 1
        {
            let mut headers = Vec::<FakeHeader>::new();
            headers.push(FakeHeader::from_field(table::Field {
                name: ":method".to_owned(),
                value: "GET".to_owned()
            }));
            headers.push(FakeHeader::from_field(table::Field {
                name: ":scheme".to_owned(),
                value: "http".to_owned()
            }));
            headers.push(FakeHeader::from_field(table::Field {
                name: ":path".to_owned(),
                value: "/".to_owned()
            }));
            headers.push(FakeHeader::from_field(table::Field {
                name: ":authority".to_owned(),
                value: "www.example.com".to_owned()
            }));

            let encoded = pack::pack(headers.iter(), &mut encoding_context, use_huffman_coding);
            assert_eq!(request_one_hex_dump, to_hex_dump(encoded.as_slice()));

            let mut decoded = unpack::UnpackedHeaders::<FakeHeader>::new();
            unpack::unpack(&encoded, &mut decoding_context, &mut decoded);

            // assert the decoded headers.
            assert_headers(&headers, &decoded.headers);
            
            // assert the dynamic table.
            assert_eq!(57, decoding_context.size());
            assert_table_entry(&decoding_context, 62, ":authority", "www.example.com");

            assert_contexts_equal(&decoding_context, &encoding_context);
        }

        // Request 2
        {
            let mut headers = Vec::<FakeHeader>::new();
            headers.push(FakeHeader::from_field(table::Field {
                name: ":method".to_owned(),
                value: "GET".to_owned()
            }));
            headers.push(FakeHeader::from_field(table::Field {
                name: ":scheme".to_owned(),
                value: "http".to_owned()
            }));
            headers.push(FakeHeader::from_field(table::Field {
                name: ":path".to_owned(),
                value: "/".to_owned()
            }));
            headers.push(FakeHeader::from_field(table::Field {
                name: ":authority".to_owned(),
                value: "www.example.com".to_owned()
            }));
            headers.push(FakeHeader::from_field(table::Field {
                name: "cache-control".to_owned(),
                value: "no-cache".to_owned()
            }));

            let encoded = pack::pack(headers.iter(), &mut encoding_context, use_huffman_coding);
            assert_eq!(request_two_hex_dump, to_hex_dump(encoded.as_slice()));

            let mut decoded = unpack::UnpackedHeaders::<FakeHeader>::new();
            unpack::unpack(&encoded, &mut decoding_context, &mut decoded);

            // assert the decoded headers.
            assert_headers(&headers, &decoded.headers);
            
            // assert the dynamic table.
            assert_eq!(110, decoding_context.size());
            assert_table_entry(&decoding_context, 62, "cache-control", "no-cache");
            assert_table_entry(&decoding_context, 63, ":authority", "www.example.com");

            assert_contexts_equal(&decoding_context, &encoding_context);
        }

        // Third request
        {
            let mut headers = Vec::<FakeHeader>::new();
            headers.push(FakeHeader::from_field(table::Field {
                name: ":method".to_owned(),
                value: "GET".to_owned()
            }));
            headers.push(FakeHeader::from_field(table::Field {
                name: ":scheme".to_owned(),
                value: "https".to_owned()
            }));
            headers.push(FakeHeader::from_field(table::Field {
                name: ":path".to_owned(),
                value: "/index.html".to_owned()
            }));
            headers.push(FakeHeader::from_field(table::Field {
                name: ":authority".to_owned(),
                value: "www.example.com".to_owned()
            }));
            headers.push(FakeHeader::from_field(table::Field {
                name: "custom-key".to_owned(),
                value: "custom-value".to_owned()
            }));

            let encoded = pack::pack(headers.iter(), &mut encoding_context, use_huffman_coding);
            assert_eq!(request_three_hex_dump, to_hex_dump(encoded.as_slice()));

            let mut decoded = unpack::UnpackedHeaders::<FakeHeader>::new();
            unpack::unpack(&encoded, &mut decoding_context, &mut decoded);

            // assert the decoded headers.
            assert_headers(&headers, &decoded.headers);
            
            // assert the dynamic table.
            assert_eq!(164, decoding_context.size());
            assert_table_entry(&decoding_context, 62, "custom-key", "custom-value");
            assert_table_entry(&decoding_context, 63, "cache-control", "no-cache");
            assert_table_entry(&decoding_context, 64, ":authority", "www.example.com");

            assert_contexts_equal(&decoding_context, &encoding_context);
        }
    }

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
        section_c_three_and_four_requests(
            true,
            "8286 8441 8cf1 e3c2 e5f2 3a6b a0ab 90f4 ff",
            "8286 84be 5886 a8eb 1064 9cbf",
            "8287 85bf 4088 25a8 49e9 5ba9 7d7f 8925 a849 e95b b8e8 b4bf"
        );
    }

    fn section_c_five_and_six_responses(
        use_huffman_coding: bool,
        response_one_hex_dump: &str,
        response_two_hex_dump: &str,
        response_three_hex_dump: &str
    ) {
        let hpack = HPack::new();
        let mut encoding_context = hpack.new_send_context();
        let mut decoding_context = hpack.new_recv_context();

        // This example uses a dynamic table with max size 256 bytes, so that some evictions occur.
        encoding_context.set_max_size(256);
        decoding_context.set_max_size(256);

        // Response 1
        {
            let mut headers = Vec::<FakeHeader>::new();
            headers.push(FakeHeader::from_field(table::Field {
                name: ":status".to_owned(),
                value: "302".to_owned()
            }));
            headers.push(FakeHeader::from_field(table::Field {
                name: "cache-control".to_owned(),
                value: "private".to_owned()
            }));
            headers.push(FakeHeader::from_field(table::Field {
                name: "date".to_owned(),
                value: "Mon, 21 Oct 2013 20:13:21 GMT".to_owned()
            }));
            headers.push(FakeHeader::from_field(table::Field {
                name: "location".to_owned(),
                value: "https://www.example.com".to_owned()
            }));
            
            let encoded = pack::pack(headers.iter(), &mut encoding_context, use_huffman_coding);

            assert_eq!(response_one_hex_dump, to_hex_dump(encoded.as_slice()));

            let mut decoded = unpack::UnpackedHeaders::<FakeHeader>::new();
            unpack::unpack(encoded.as_slice(), &mut decoding_context, &mut decoded);

            // assert the decoded headers.
            assert_headers(&headers, &decoded.headers);
            
            // assert the dynamic table.
            assert_eq!(222, decoding_context.size());
            assert_table_entry(&decoding_context, 62, "location", "https://www.example.com");
            assert_table_entry(&decoding_context, 63, "date", "Mon, 21 Oct 2013 20:13:21 GMT");
            assert_table_entry(&decoding_context, 64, "cache-control", "private");
            assert_table_entry(&decoding_context, 65, ":status", "302");

            assert_contexts_equal(&decoding_context, &encoding_context);
        }

        // Response 2
        {
            let mut headers = Vec::<FakeHeader>::new();
            headers.push(FakeHeader::from_field(table::Field {
                name: ":status".to_owned(),
                value: "307".to_owned()
            }));
            headers.push(FakeHeader::from_field(table::Field {
                name: "cache-control".to_owned(),
                value: "private".to_owned()
            }));
            headers.push(FakeHeader::from_field(table::Field {
                name: "date".to_owned(),
                value: "Mon, 21 Oct 2013 20:13:21 GMT".to_owned()
            }));
            headers.push(FakeHeader::from_field(table::Field {
                name: "location".to_owned(),
                value: "https://www.example.com".to_owned()
            }));

            let encoded = pack::pack(headers.iter(), &mut encoding_context, use_huffman_coding);

            assert_eq!(response_two_hex_dump, to_hex_dump(encoded.as_slice()));

            let mut decoded = unpack::UnpackedHeaders::<FakeHeader>::new();
            unpack::unpack(encoded.as_slice(), &mut decoding_context, &mut decoded);

            // assert the decoded headers.
            assert_headers(&headers, &decoded.headers);
            
            // assert the dynamic table.
            assert_eq!(222, decoding_context.size());
            assert_table_entry(&decoding_context, 62, ":status", "307");
            assert_table_entry(&decoding_context, 63, "location", "https://www.example.com");
            assert_table_entry(&decoding_context, 64, "date", "Mon, 21 Oct 2013 20:13:21 GMT");
            assert_table_entry(&decoding_context, 65, "cache-control", "private");

            assert_contexts_equal(&decoding_context, &encoding_context);
        }

        // Response 3
        {
            let mut headers = Vec::<FakeHeader>::new();
            headers.push(FakeHeader::from_field(table::Field {
                name: ":status".to_owned(),
                value: "200".to_owned()
            }));
            headers.push(FakeHeader::from_field(table::Field {
                name: "cache-control".to_owned(),
                value: "private".to_owned()
            }));
            headers.push(FakeHeader::from_field(table::Field {
                name: "date".to_owned(),
                value: "Mon, 21 Oct 2013 20:13:22 GMT".to_owned()
            }));
            headers.push(FakeHeader::from_field(table::Field {
                name: "location".to_owned(),
                value: "https://www.example.com".to_owned()
            }));
            headers.push(FakeHeader::from_field(table::Field {
                name: "content-encoding".to_owned(),
                value: "gzip".to_owned()
            }));
            headers.push(FakeHeader::from_field(table::Field {
                name: "set-cookie".to_owned(),
                value: "foo=ASDJKHQKBZXOQWEOPIUAXQWEOIU; max-age=3600; version=1".to_owned()
            }));

            let encoded = pack::pack(headers.iter(), &mut encoding_context, use_huffman_coding);

            assert_eq!(response_three_hex_dump, to_hex_dump(encoded.as_slice()));

            let mut decoded = unpack::UnpackedHeaders::<FakeHeader>::new();
            unpack::unpack(encoded.as_slice(), &mut decoding_context, &mut decoded);

            // assert the decoded headers.
            assert_headers(&headers, &decoded.headers);
            
            // assert the dynamic table.
            assert_eq!(215, decoding_context.size());
            assert_table_entry(&decoding_context, 62, "set-cookie", "foo=ASDJKHQKBZXOQWEOPIUAXQWEOIU; max-age=3600; version=1");
            assert_table_entry(&decoding_context, 63, "content-encoding", "gzip");
            assert_table_entry(&decoding_context, 64, "date", "Mon, 21 Oct 2013 20:13:22 GMT");

            assert_contexts_equal(&decoding_context, &encoding_context);
        }
    }

    // See C.5
    #[test]
    pub fn multiple_responses_without_huffman_coding() {
        section_c_five_and_six_responses(
            false,
            "4803 3330 3258 0770 7269 7661 7465 611d 4d6f 6e2c 2032 3120 4f63 7420 3230 3133 2032 303a 3133 3a32 3120 474d 546e 1768 7474 7073 3a2f 2f77 7777 2e65 7861 6d70 6c65 2e63 6f6d",
            "4803 3330 37c1 c0bf",
            "88c1 611d 4d6f 6e2c 2032 3120 4f63 7420 3230 3133 2032 303a 3133 3a32 3220 474d 54c0 5a04 677a 6970 7738 666f 6f3d 4153 444a 4b48 514b 425a 584f 5157 454f 5049 5541 5851 5745 4f49 553b 206d 6178 2d61 6765 3d33 3630 303b 2076 6572 7369 6f6e 3d31"
        )
    }

    // See C.6
    #[test]
    pub fn multiple_responses_with_huffman_coding() {
        section_c_five_and_six_responses(
            true,
            "4882 6402 5885 aec3 771a 4b61 96d0 7abe 9410 54d4 44a8 2005 9504 0b81 66e0 82a6 2d1b ff6e 919d 29ad 1718 63c7 8f0b 97c8 e9ae 82ae 43d3",
            "4883 640e ffc1 c0bf",
            "88c1 6196 d07a be94 1054 d444 a820 0595 040b 8166 e084 a62d 1bff c05a 839b d9ab 77ad 94e7 821d d7f2 e6c7 b335 dfdf cd5b 3960 d5af 2708 7f36 72c1 ab27 0fb5 291f 9587 3160 65c0 03ed 4ee5 b106 3d50 07"
        )
    }
}
