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

#[derive(Debug)]
pub struct EncodedNumber {
    pub prefix: u8,
    pub rest: Option<Vec<u8>>
}

#[derive(Debug)]
pub struct DecodedNumber {
    pub num: u32,
    pub octets_read: usize
}

// TODO is u32 the right type to use? probably want to allow number to get as big as they can before they're capped.
// maybe use usize instead?

// see above, usize is pointer sized (32 bits in x86 and 64 bits on x86_64) which would mean the server adapts to the system it runs on?
// and the server can use the max sizes provided by rust to decide which values are too large to decode at runtime.

// n here is N in the hpack encoding instructions.
// n must lie between 1 and 8 inclusive
pub fn encode(num: u32, n: u8) -> EncodedNumber {
    if num < (1 << n) - 1 {
        return EncodedNumber {
            prefix: num as u8,
            rest: None
        }
    }

    let two_pow_n_minus_one = (1 << n) - 1;

    let mut rest = Vec::new();
    let mut _num = num - two_pow_n_minus_one;
    while _num >= 128 {
        rest.push(((_num % 128) + 128) as u8);
        _num = _num / 128;
    }
    rest.push(_num as u8);

    EncodedNumber {
        prefix: two_pow_n_minus_one as u8,
        rest: Some(rest)
    }
}

// TODO must reject encodings which overflow integer on decode.

// octets must have length at least 1
// n must be between 1 and 8 inclusive
pub fn decode(octets: &mut Peekable<Iter<u8>>, n: u8) -> DecodedNumber {
    // turn off bits which should not be checked.
    let mut num = (*octets.next().unwrap() & (255 >> (8 - n))) as u32;
    if num < (1 << n) - 1 {
        return DecodedNumber {
            num: num,
            octets_read: 1   
        };
    }

    // We have read the prefix already, now count how many octets are in the rest list.
    let mut octets_read = 1;

    let mut m = 0;
    while let Some(&octet) = octets.peek() {
        num = num + (octet & 127) as u32 * (1 << m);
        m = m + 7;

        octets_read += 1;

        octets.next();

        if octet & 128 != 128 {break;}
    }

    DecodedNumber {
        num: num,
        octets_read: octets_read
    }
}

#[cfg(test)]
mod tests {
    use super::{encode, decode};

    // slightly clumsy function to print the bits of a u8.
    // it's useful even if it's bad :)
    #[allow(unused)]
    pub fn print_binary(octet: u8) {
        let mut bits = [0, 0, 0, 0, 0, 0, 0, 0];
        for i in 0..8 {
            let filter = 2i32.pow(i as u32) as u8;
            bits[7 - i] = (octet & filter == filter) as u8;
        }

        println!("{:?}", bits);
    }

    // See example C.1.1 of hpack instructions.
    #[test]
    fn encdode_in_prefix() {
        let en = encode(10, 5);

        assert_eq!(10, en.prefix);
        assert!(en.rest.is_none());
    }

    #[test]
    fn decode_prefix_only() {
        let en = encode(10, 5);
        let octets = vec!(en.prefix);

        let dn = decode(&mut octets.iter().peekable(), 5);
        assert_eq!(1, dn.octets_read);
        assert_eq!(10, dn.num);
    }

    // See example C.1.2 of hpack instructions.
    #[test]
    fn encode_using_rest() {
        let en = encode(1337, 5);

        assert_eq!(31, en.prefix);
        assert!(en.rest.is_some());
        let rest = en.rest.unwrap();
        assert_eq!(154, rest[0]);
        assert_eq!(10, rest[1]);
    }

    #[test]
    fn decode_using_rest() {
        let en = encode(1337, 5);
        let mut octets = vec!(en.prefix);
        octets.extend(en.rest.unwrap());

        let de = decode(&mut octets.iter().peekable(), 5);
        assert_eq!(3, de.octets_read);
        assert_eq!(1337, de.num);
    }

    // See example C.1.3 of hpack instructions.
    #[test]
    fn encode_starting_at_octet_boundary() {
        let en = encode(42, 8);

        assert_eq!(42, en.prefix);
        assert!(en.rest.is_none());
    }

    #[test]
    fn decode_starting_at_octet_boundary() {
        let en = encode(42, 8);
        let octets = vec!(en.prefix);

        let dn = decode(&mut octets.iter().peekable(), 8);
        assert_eq!(1, dn.octets_read);
        assert_eq!(42, dn.num);
    }
}
