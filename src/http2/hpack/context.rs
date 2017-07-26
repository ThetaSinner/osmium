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

use http2::hpack::table;
use http2::hpack::table::Field;

// Notice that the static table is a reference to a single static table instance. 
// That is, there is a single instance of the static table in the program.
// The dynamic table belongs to this context.
pub struct Context {
    static_table: &'static table::Table,
    dynamic_table: table::Table
}

impl Context {
    pub fn insert(&mut self, field: Field) {
        self.dynamic_table.push_front(field);
    }

    pub fn get(&self, index: usize) -> Option<&Field> {
        if index < self.static_table.len() {
            self.static_table.get(index)
        }
        else {
            self.dynamic_table.get(index)
        }
    }

    pub fn find_field(&self, field: &Field) -> Option<(usize, bool)> {
        let opt_index = self.static_table.find_field(field);

        if let Some((_, true)) = opt_index {
            opt_index
        }
        else {
            self.dynamic_table.find_field(field)
        }
    }

    pub fn set_max_size(&mut self, max_size: usize) {
        self.dynamic_table.set_max_size(max_size);
    }
}
