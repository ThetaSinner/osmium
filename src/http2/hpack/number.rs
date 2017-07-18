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

pub struct EncodedNumber {
    prefix: u8,
    rest: Option<Vec<u8>>
}

// n here is N in the hpack encoding instructions.
// n must lie between 1 and 8 inclusive
pub fn encode(num: i32, n: u8) -> EncodedNumber {
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

// octets must have length at least 1
// n must be between 1 and 8 inclusive
pub fn decode(octets: Vec<u8>, n: u8) -> i32 {
    // turn off bits which should not be checked.
    let mut num = (octets[0] & (255 >> (8 - n))) as i32;
    if num < (1 << n) - 1 {
        return num;
    }

    let mut m = 0;
    for i in 1..octets.len() {
        let octet = octets[i];

        num = num + (octet & 127) as i32 * (1 << m);
        m = m + 7;

        if octet & 128 != 128 {break;}
    }

    num
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
    fn decode_for_encdode_in_prefix() {
        let en = encode(10, 5);
        let octets = vec!(en.prefix);
        let num = decode(octets, 5);

        assert_eq!(10, num);
    }

    // See example C.1.2 of hpack instructions.
    #[test]
    fn encdode_using_rest() {
        let en = encode(1337, 5);

        assert_eq!(31, en.prefix);
        assert!(en.rest.is_some());
        let rest = en.rest.unwrap();
        assert_eq!(154, rest[0]);
        assert_eq!(10, rest[1]);
    }

    #[test]
    fn decode_for_encdode_using_rest() {
        let en = encode(1337, 5);
        let mut octets = vec!(en.prefix);
        octets.extend(en.rest.unwrap());

        let num = decode(octets, 5);
        assert_eq!(1337, num);
    }

    // See example C.1.3 of hpack instructions.
    #[test]
    fn encdode_starting_at_octet_boundary() {
        let en = encode(42, 8);

        assert_eq!(42, en.prefix);
        assert!(en.rest.is_none());
    }

    #[test]
    fn decode_for_encdode_starting_at_octet_boundary() {
        let en = encode(42, 8);
        let octets = vec!(en.prefix);

        print_binary(octets[0]);

        let num = decode(octets, 8);
        assert_eq!(42, num);
    }
}
