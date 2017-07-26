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
use std::slice::Iter;

// osmium
use http2::hpack::number;

#[derive(Debug)]
pub struct DecodedString {
    pub string: String,
    pub octets_read: usize
}

pub fn encode(string: String, use_huffman_coding: bool) -> Vec<u8> {
    let mut length = number::encode(string.len() as i32, 7);

    // As the string length only uses the first 7 bits of its prefix
    // we can use the 8th bit as a flag for Huffman coding.
    // If Huffman coding is not being used then leave the bit set to 0,
    // otherwise set the bit to 1.
    if use_huffman_coding {
        length.prefix |= 128;
    }

    let mut result = vec!(length.prefix);
    if let Some(rest) = length.rest {
        result.extend(rest);
    }
    result.extend(string.as_bytes().to_vec());

    result
}

pub fn decode(octets: &mut Iter<u8>) -> DecodedString {
    let mut use_huffman_coding = true;
    if **octets.peekable().peek().unwrap() & 128 == 0 {
        use_huffman_coding = false;
    }

    // TODO the huffman coding is now implemented and needs to be used here.
    if use_huffman_coding {
        panic!("Cannot decode Huffman encoded string");
    }

    let dn = number::decode(octets, 7);

    let mut str_bytes = Vec::new();
    let mut take_string = octets.take(dn.octets_read);
    while let Some(&str_byte) = take_string.next() {
        str_bytes.push(str_byte);
    }

    DecodedString {
        octets_read: dn.octets_read + str_bytes.len(),
        string: String::from_utf8(str_bytes).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::{encode, decode};
    use super::number;

    #[test]
    fn encode_hello_world() {
        let result = encode("Hello, World!".to_owned(), false);

        // string length = 13, length stored in one octet, so total 14.
        assert_eq!(14, result.len());

        // first octet is length of the string.
        assert_eq!(13, result[0]);

        assert_eq!("Hello, World!", String::from_utf8(result[1..].to_vec()).unwrap());
    }

    #[test]
    fn decode_for_encode_hello_world() {
        let es = encode("Hello, World!".to_owned(), false);

        let ds = decode(es.as_slice());
        assert_eq!(14, ds.octets_read);
        assert_eq!("Hello, World!", ds.string);
    }

    #[test]
    fn encode_string_which_has_length_encoding_too_long_for_prefix() {
        let test_string = "this is an excessively long string which overflows the prefix, to check that the 'rest' length bytes are included in the string encoding".to_owned();
        let result = encode(test_string.to_owned(), false);


        // assert the total length of the string encoding
        assert_eq!(138, result.len());
        
        // assert that the huffman coding flag is off
        assert!(result[0] & 128 == 0);

        // assert the string length encoding
        let dn = number::decode(&vec!(result[0], result[1]), 7);
        assert_eq!(2, dn.octets_read);
        assert_eq!(test_string.len() as i32, dn.num);

        // assert the string
        assert_eq!(test_string, String::from_utf8(result[2..].to_vec()).unwrap());
    }

    #[test]
    fn decode_for_encode_string_which_has_length_encoding_too_long_for_prefix() {
        let test_string = "this is an excessively long string which overflows the prefix, to check that the 'rest' length bytes are included in the string encoding".to_owned();
        let es = encode(test_string.to_owned(), false);

        let ds = decode(es.as_slice());
        assert_eq!(2 + test_string.len(), ds.octets_read as usize);
        assert_eq!(test_string, ds.string);
    }
}
