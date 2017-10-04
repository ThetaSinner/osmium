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
use http2::hpack;

/// Represents and hpack context, as defined in hpack section 2.2.
///
/// The context consists two indexing tables, see hpack section 2.3. The tables are
/// a static table and a dynamic table, see hpack sections 2.3.1 and 2.3.2.
///
/// The two tables are accesed in a single address space, as defined in hpack section 2.3.3.
/// The context structure provides an interface to this single address space.
///
/// Note that this context has a reference to a single instance static table which is required
/// to live at least as long as this context instance. The dynamic table however belongs to 
/// this context.
pub struct Context<'a> {
    static_table: &'a table::Table,
    dynamic_table: table::Table
}

// TODO Field type needs to go and be replaced by Header types.

impl<'a> Context<'a> {
    /// Create a new context object with a reference to the given static table.
    pub fn new(static_table: &'a table::Table) -> Self {
        Context {
            static_table: static_table,
            dynamic_table: table::Table::new()
        }
    }

    /// Inserts a header into the dynamic table. As per section 2.3.3, insertion is at the front
    /// of the dynamic table. Equivalently, the insertion point is after the end of the static table.
    pub fn insert(&mut self, field: Field) {
        self.dynamic_table.push_front(field);
    }

    /// Get by index, where index is from 1 to static_table length + dynamic table length/
    /// See hpack section 2.3.3 on the single address space.
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

    /// Returns the best match with the lowest matching index, or none otherwise.
    /// The tuple returned contains the index matched and a flag indicating whether the 
    /// header value matches. If there is a possible match with this flag true, then it is 
    /// prefered over a match with the flag false.
    /// The index returned refers to the single address space.
    pub fn find_field(&self, field: &Field) -> Option<(usize, bool)> {
        let opt_static_index = self.static_table.find_field(field);

        // TODO how very untidy.

        if let Some((_, true)) = opt_static_index {
            // the static match is optimal, return it
            trace!("Found optimal match in static table at index {}", opt_static_index.unwrap().0);
            opt_static_index
        }
        else {
            let opt_dymamic_index = self.dynamic_table.find_field(field);

            if let Some((index, true)) = opt_dymamic_index {
                // the dynamic mathc is optimal return it
                trace!("Found optimal match in dynamic table at index {}", index);
                Some((index + hpack::STATIC_TABLE_LENGTH, true))
            }
            else {
                // neither match is optimal, return the lowest index which is some
                if opt_static_index.is_some() {
                    trace!("No optimal match, prefering static index");
                    opt_static_index
                }
                else if let Some((index, with_value)) = opt_dymamic_index {
                    trace!("No optimal match, and there is no matching static index");
                    Some((index + hpack::STATIC_TABLE_LENGTH, with_value))
                }
                else {
                    None
                }
            }
        }
    }

    /// The size of the dynamic table in bytes. See hpack section 4.1
    pub fn size(&self) -> usize {
        self.dynamic_table.get_size()
    }

    /// Set the maximum size the dynamic table is permitted to use. This value must be
    /// less than SETTINGS_HEADER_TABLE_SIZE, see http2 6.5.2.
    ///
    /// Note that reducing the size of the dynamic table may cause entry eviction, as per
    /// hpack section 4.3.
    ///
    // TODO the hpack spec does not define how to handle an error (value larger than size setting),
    // so this code should be modified after reading the http2 spec.
    // UPDATE the max size can't just be set, the decoder needs to be notified if the encoder size 
    // is updated. As for handling updating to be too large, that is only going to happen if this
    // application does something stupid, so ending the connection with NO_ERROR or similar is 
    // probably the way forward.
    pub fn set_max_size(&mut self, max_size: usize) {
        self.dynamic_table.set_max_size(max_size);
    }

    /// Informs the context of a change to SETTINGS_HEADER_TABLE_SIZE, see http2 6.5.2
    /// 
    // TODO Create two traits, one for send and one for recv contexts. This will then only
    // be on the send context.
    pub fn inform_max_size_setting_changed(&mut self, max_size_setting: u32) {
        // TODO the pack code doesn't handle sending a size update, so there's no point
        // writing this code until the pack code is modified. In which case it makes
        // sense to handle the TODO above.
        // Look at the pen of pain I opened...
    }
}
