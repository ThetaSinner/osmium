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
pub struct Context<'a> {
    static_table: &'a table::Table,
    dynamic_table: table::Table
}

// TODO Field type needs to go and be replaced by Header types.

impl<'a> Context<'a> {
    pub fn new(static_table: &'a table::Table) -> Self {
        Context {
            static_table: static_table,
            dynamic_table: table::Table::new()
        }
    }

    pub fn insert(&mut self, field: Field) {
        self.dynamic_table.push_front(field);
    }

    /// Get by index, where index is from 1 to static_table length + dynamic table length
    pub fn get(&self, index: usize) -> Option<&Field> {
        // check that the input index refers to a table index rather than a vector index.
        assert!(1 <= index);

        let table_index = index - 1;
        if table_index < self.static_table.len() {
            self.static_table.get(table_index)
        }
        else {
            self.dynamic_table.get(table_index - self.static_table.len())
        }
    }

    pub fn find_field(&self, field: &Field) -> Option<(usize, bool)> {
        let opt_static_index = self.static_table.find_field(field);

        // TODO how very untidy.

        if let Some((_, true)) = opt_static_index {
            // the static match is optimal, return it
            opt_static_index
        }
        else {
            let opt_dymamic_index = self.dynamic_table.find_field(field);

            if let Some((_, true)) = opt_dymamic_index {
                // the dynamic mathc is optimal return it
                opt_dymamic_index
            }
            else {
                // neither match is optimal, return the lowest index which is some
                if opt_static_index.is_some() {
                    opt_static_index
                }
                else {
                    opt_dymamic_index
                }
            }
        }
    }

    // The size of the dynamic table
    pub fn size(&self) -> usize {
        self.dynamic_table.get_size()
    }

    pub fn set_max_size(&mut self, max_size: usize) {
        self.dynamic_table.set_max_size(max_size);
    }
}
