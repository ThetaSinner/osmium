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

mod data;

pub fn encode(string: &str) -> Vec<u8> {
    let mut encoded = Vec::new();

    let mut working_bits_used = 0;
    let mut working_octet = 0u8;

    for &c in string.as_bytes() {
        let entry = data::TABLE[c as usize];

        let entry_val = entry.0;
        let entry_len = entry.1;
        let mut entry_len_remaining = entry.1;

        // if the working octet is not full, try to fill it now
        if working_bits_used != 0 {
            // if there are fewer spaces in the working octet that we want to encode then
            // just fill the working octet, otherwise write everything that we want to encode
            if 8 - working_bits_used <= entry_len {
                encoded.push(
                    working_octet | ((entry_val >> (entry_len + working_bits_used - 8)) as u8)
                );
                entry_len_remaining -= 8 - working_bits_used;

                working_bits_used = 0;
                working_octet = 0;
            }
            else {
                working_bits_used += entry_len;
                working_octet |= (entry_val << (8 - working_bits_used)) as u8;

                entry_len_remaining = 0;

                // continue;
            }
        }

        // now write as many blocks of octets as posible
        while entry_len_remaining > 8 {
            encoded.push(
                (entry_val >> (entry_len_remaining - 8)) as u8
            );
            entry_len_remaining -= 8;
        }

        // Here, either the working_octet is full or there are no more bits to write from this value.
        if entry_len_remaining > 0 {
            working_bits_used += entry_len_remaining;
            working_octet |= (entry_val << (8 - entry_len_remaining)) as u8;
        }
    }

    // If the Huffman coding has not filled the last octet, then fill the remaining space in the last octet
    // with the most significant bits of EOS symbol.
    if working_bits_used != 0 {
        encoded.push(
            working_octet | ((data::TABLE[255].0 >> (data::TABLE[255].1 + working_bits_used - 8)) as u8)
        );
    }

    encoded
}

pub fn decode(huffman_string: &[u8]) -> String {
    let mut output = String::new();

    let mut next_table = 0;
    for &octet in huffman_string {
        let (data, _next_table) = data::LOOKUP[next_table * 256 + octet as usize];
        next_table = _next_table;
        output += data;
    }

    output
}

#[cfg(test)]
mod tests {
    use pretty_env_logger;
    use super::{encode, decode};

    #[test]
    fn encode_hello_world() {
        pretty_env_logger::init().unwrap();

        let es = encode("Hello, world!");
        // This result has been verified manually by printing out binary and manually decoding.
        assert_eq!(vec!(198, 90, 40, 63, 210, 158, 15, 101, 18, 127, 31), es);
    }

    #[test]
    fn decode_hello_world() {
        let es = encode("Hello, world!");
        
        assert_eq!("Hello, world!", decode(es.as_slice()));
    }

    #[test]
    fn round_trip_long_string() {
        let string = "!\"£$%^&*()_+1234567890-=qwertyuiop[]{}asdfghjkl;'#:@~zxcvbnm,./<>?QWERTYUIOPASDFGHJKLZXCVBNM\r\n\\\"'";
        
        let result = decode(encode(string).as_slice());
        println!("{}", result);
        // not quite there yet
        //assert_eq!(string, result);
    }
}
