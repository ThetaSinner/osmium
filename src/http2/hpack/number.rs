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

#[cfg(test)]
mod tests {
    use super::encode;

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

    // See example C.1.3 of hpack instructions.
    #[test]
    fn encdode_starting_at_octet_boundary() {
        let en = encode(42, 8);

        assert_eq!(42, en.prefix);
        assert!(en.rest.is_none());
    }
}
