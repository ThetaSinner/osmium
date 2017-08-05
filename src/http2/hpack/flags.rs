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

//! Flags which are used for settings and reading bits used as hpack 
//! markers.

/// The bit pattern `10000000`, used to match the prefix `1`
pub const INDEXED_HEADER_FLAG: u8 = 0x80;

/// The bit pattern `01000000` used to match the prefix `01`
pub const LITERAL_WITH_INDEXING_FLAG: u8 = 0x40;
/// The bit pattern `11110000`, used to match the prefix `0000`
pub const LITERAL_WITHOUT_INDEXING_FLAG: u8 = 0xf0;
/// The bit pattern `00010000`, used to match the prefix `0001`
pub const LITERAL_NEVER_INDEX_FLAG: u8 = 0x10;

/// The bit pattern `00100000`, used to match the prefix `001`
pub const SIZE_UPDATE_FLAG: u8 = 0x20;
