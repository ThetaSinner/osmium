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
use std::iter::Peekable;

// osmium
use http2::hpack::number;
use http2::hpack::huffman;

#[derive(Debug)]
pub struct DecodedString {
    pub string: String,
    pub octets_read: usize
}

pub fn encode(string: String, use_huffman_coding: bool) -> Vec<u8> {
    // TODO the length of the string to encode should be capped.

    let mut length = number::encode(string.len() as u32, 7);

    // As the string length only uses the first 7 bits of its prefix
    // we can use the 8th bit as a flag for Huffman coding.
    // If Huffman coding is not being used then leave the bit set to 0,
    // otherwise set the bit to 1.
    if use_huffman_coding {
        length.prefix |= 128;
    }

    // encode the string length in the representation.
    let mut result = vec!(length.prefix);
    if let Some(rest) = length.rest {
        result.extend(rest);
    }

    // encode the actual string in the representation, either plain or compressed with huffman coding.
    if use_huffman_coding {
        result.extend(huffman::encode(string.as_str()));
    }
    else {
        result.extend(string.as_bytes().to_vec());
    }

    result
}

pub fn decode(octets: &mut Peekable<Iter<u8>>) -> DecodedString {
    let mut use_huffman_coding = true;
    if *octets.peek().unwrap() & 128 == 0 {
        use_huffman_coding = false;
    }

    let dn = number::decode(octets, 7);

    let mut take_string = octets.take(dn.num as usize);
    let string = if use_huffman_coding {
        // this is horrible... really nasty.
        let taken: Vec<u8> = take_string.map(|&x| {x}).collect();
        huffman::decode(taken.as_slice())
    }
    else {
        let mut str_bytes = Vec::new();
        while let Some(&str_byte) = take_string.next() {
            str_bytes.push(str_byte);
        }

        String::from_utf8(str_bytes).unwrap()
    };

    DecodedString {
        octets_read: dn.octets_read + dn.num as usize,
        string: string
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
    fn decode_hello_world() {
        let es = encode("Hello, World!".to_owned(), false);

        let ds = decode(&mut es.iter().peekable());
        assert_eq!(14, ds.octets_read);
        assert_eq!("Hello, World!", ds.string);
    }

    #[test]
    fn round_trip_huffman_hello_world() {
        let result = encode("Hello, World!".to_owned(), true);

        let original = decode(&mut result.iter().peekable());

        assert_eq!("Hello, World!", original.string);
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
        let dn = number::decode(&mut vec!(result[0], result[1]).iter().peekable(), 7);
        assert_eq!(2, dn.octets_read);
        assert_eq!(test_string.len(), dn.num as usize);

        // assert the string
        assert_eq!(test_string, String::from_utf8(result[2..].to_vec()).unwrap());
    }

    #[test]
    fn decode_string_which_has_length_encoding_too_long_for_prefix() {
        let test_string = "this is an excessively long string which overflows the prefix, to check that the 'rest' length bytes are included in the string encoding".to_owned();
        let es = encode(test_string.to_owned(), false);

        let ds = decode(&mut es.iter().peekable());
        assert_eq!(2 + test_string.len(), ds.octets_read as usize);
        assert_eq!(test_string, ds.string);
    }
}
