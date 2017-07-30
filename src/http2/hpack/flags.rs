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

pub static INDEXED_HEADER_FLAG: u8 = 0x80;

pub static LITERAL_WITH_INDEXING_FLAG: u8 = 0x40;
pub static LITERAL_WITHOUT_INDEXING_FLAG: u8 = 0xf0;
pub static LITERAL_NEVER_INDEX_FLAG: u8 = 0x10;

pub static SIZE_UPDATE_FLAG: u8 = 0x20;
