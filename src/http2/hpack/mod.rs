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
         
        static_table.push_front(Field{name: String::from(":authority"), value: String::from("")});
        static_table.push_front(Field{name: String::from(":method"), value: String::from("GET")});
        static_table.push_front(Field{name: String::from(":method"), value: String::from("POST")});
        static_table.push_front(Field{name: String::from(":path"), value: String::from("/")});
        static_table.push_front(Field{name: String::from(":path"), value: String::from("/index.html")});
        static_table.push_front(Field{name: String::from(":scheme"), value: String::from("http")});
        static_table.push_front(Field{name: String::from(":scheme"), value: String::from("https")});
        static_table.push_front(Field{name: String::from(":status"), value: String::from("200")});
        static_table.push_front(Field{name: String::from(":status"), value: String::from("204")});
        static_table.push_front(Field{name: String::from(":status"), value: String::from("206")});
        static_table.push_front(Field{name: String::from(":status"), value: String::from("304")});
        static_table.push_front(Field{name: String::from(":status"), value: String::from("400")});
        static_table.push_front(Field{name: String::from(":status"), value: String::from("404")});
        static_table.push_front(Field{name: String::from(":status"), value: String::from("500")});
        static_table.push_front(Field{name: String::from("accept-charset"), value: String::from("")});
        static_table.push_front(Field{name: String::from("accept-encoding"), value: String::from("gzip, deflate")});
        static_table.push_front(Field{name: String::from("accept-language"), value: String::from("")});
        static_table.push_front(Field{name: String::from("accept-ranges"), value: String::from("")});
        static_table.push_front(Field{name: String::from("accept"), value: String::from("")});
        static_table.push_front(Field{name: String::from("access-control-allow-origin"), value: String::from("")});
        static_table.push_front(Field{name: String::from("age"), value: String::from("")});
        static_table.push_front(Field{name: String::from("allow"), value: String::from("")});
        static_table.push_front(Field{name: String::from("authorization"), value: String::from("")});
        static_table.push_front(Field{name: String::from("cache-control"), value: String::from("")});
        static_table.push_front(Field{name: String::from("content-disposition"), value: String::from("")});
        static_table.push_front(Field{name: String::from("content-encoding"), value: String::from("")});
        static_table.push_front(Field{name: String::from("content-language"), value: String::from("")});
        static_table.push_front(Field{name: String::from("content-length"), value: String::from("")});
        static_table.push_front(Field{name: String::from("content-location"), value: String::from("")});
        static_table.push_front(Field{name: String::from("content-range"), value: String::from("")});
        static_table.push_front(Field{name: String::from("content-type"), value: String::from("")});
        static_table.push_front(Field{name: String::from("cookie"), value: String::from("")});
        static_table.push_front(Field{name: String::from("date"), value: String::from("")});
        static_table.push_front(Field{name: String::from("etag"), value: String::from("")});
        static_table.push_front(Field{name: String::from("expect"), value: String::from("")});
        static_table.push_front(Field{name: String::from("expires"), value: String::from("")});
        static_table.push_front(Field{name: String::from("from"), value: String::from("")});
        static_table.push_front(Field{name: String::from("host"), value: String::from("")});
        static_table.push_front(Field{name: String::from("if-match"), value: String::from("")});
        static_table.push_front(Field{name: String::from("if-modified-since"), value: String::from("")});
        static_table.push_front(Field{name: String::from("if-none-match"), value: String::from("")});
        static_table.push_front(Field{name: String::from("if-range"), value: String::from("")});
        static_table.push_front(Field{name: String::from("if-unmodified-since"), value: String::from("")});
        static_table.push_front(Field{name: String::from("last-modified"), value: String::from("")});
        static_table.push_front(Field{name: String::from("link"), value: String::from("")});
        static_table.push_front(Field{name: String::from("location"), value: String::from("")});
        static_table.push_front(Field{name: String::from("max-forwards"), value: String::from("")});
        static_table.push_front(Field{name: String::from("proxy-authenticate"), value: String::from("")});
        static_table.push_front(Field{name: String::from("proxy-authorization"), value: String::from("")});
        static_table.push_front(Field{name: String::from("range"), value: String::from("")});
        static_table.push_front(Field{name: String::from("referer"), value: String::from("")});
        static_table.push_front(Field{name: String::from("refresh"), value: String::from("")});
        static_table.push_front(Field{name: String::from("retry-after"), value: String::from("")});
        static_table.push_front(Field{name: String::from("server"), value: String::from("")});
        static_table.push_front(Field{name: String::from("set-cookie"), value: String::from("")});
        static_table.push_front(Field{name: String::from("strict-transport-security"), value: String::from("")});
        static_table.push_front(Field{name: String::from("transfer-encoding"), value: String::from("")});
        static_table.push_front(Field{name: String::from("user-agent"), value: String::from("")});
        static_table.push_front(Field{name: String::from("vary"), value: String::from("")});
        static_table.push_front(Field{name: String::from("via"), value: String::from("")});
        static_table.push_front(Field{name: String::from("www-authenticate"), value: String::from("")});

        assert_eq!(61, static_table.len(), "static table should have 61 entries");

        HPack {
            static_table: static_table
        }
    }

    pub fn new_context(&self) -> Context {
        Context::new(&self.static_table)
    }
}
