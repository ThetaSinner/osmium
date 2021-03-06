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

pub trait ContextTrait<'a> {
    /// Create a new context object with a reference to the given static table.
    fn new(static_table: &'a table::Table) -> Self;

    /// Inserts a header into the dynamic table. As per section 2.3.3, insertion is at the front
    /// of the dynamic table. Equivalently, the insertion point is after the end of the static table.
    fn insert(&mut self, field: Field);

    /// Get by index, where index is from 1 to static_table length + dynamic table length/
    /// See hpack section 2.3.3 on the single address space.
    fn get(&self, index: usize) -> Option<&Field>;

    /// Returns the best match with the lowest matching index, or none otherwise.
    /// The tuple returned contains the index matched and a flag indicating whether the 
    /// header value matches. If there is a possible match with this flag true, then it is 
    /// prefered over a match with the flag false.
    /// The index returned refers to the single address space.
    fn find_field(&self, field: &Field) -> Option<(usize, bool)>;

    /// The size of the dynamic table in bytes. See hpack section 4.1
    // TODO it's not quite the size in bytes, update docs.
    fn size(&self) -> usize;

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
    fn set_max_size(&mut self, max_size: usize);
}

impl<'a> ContextTrait<'a> for Context<'a> {
    /// Create a new context object with a reference to the given static table.
    fn new(static_table: &'a table::Table) -> Self {
        Context {
            static_table: static_table,
            dynamic_table: table::Table::new()
        }
    }

    /// Inserts a header into the dynamic table. As per section 2.3.3, insertion is at the front
    /// of the dynamic table. Equivalently, the insertion point is after the end of the static table.
    fn insert(&mut self, field: Field) {
        self.dynamic_table.push_front(field);
    }

    /// Get by index, where index is from 1 to static_table length + dynamic table length/
    /// See hpack section 2.3.3 on the single address space.
    fn get(&self, index: usize) -> Option<&Field> {
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
    fn find_field(&self, field: &Field) -> Option<(usize, bool)> {
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
    fn size(&self) -> usize {
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
    fn set_max_size(&mut self, max_size: usize) {
        self.dynamic_table.set_max_size(max_size);
    }
}

pub struct SendContext<'a> {
    inner: Context<'a>,

    send_size_update: bool,
    size_update: u32
}

impl<'a> SendContext<'a> {
    /// Informs the context of a change to SETTINGS_HEADER_TABLE_SIZE, see http2 6.5.2
    pub fn inform_max_size_setting_changed(&mut self, max_size_setting: u32) {
        // TODO handle multiple max size setting updates between header encodes.

        // For now, don't do anything, just let the max size setting drive this.
        self.size_update = max_size_setting;
        self.send_size_update = true;
    }

    pub fn get_size_update(&mut self) -> Option<u32> {
        if self.send_size_update {
            self.send_size_update = false;
            return Some(self.size_update)
            // TODO note that size_update isn't cleared. Should it be, and how can it be 
            // made obvious that it has been cleared?
        }
        
        None
    }
}

impl<'a> ContextTrait<'a> for SendContext<'a> {
    fn new(static_table: &'a table::Table) -> Self {
        SendContext {
            inner: Context::new(static_table),

            send_size_update: false,
            size_update: 0
        }
    }

    fn insert(&mut self, field: Field) {
        self.inner.insert(field)
    }

    fn get(&self, index: usize) -> Option<&Field> {
        self.inner.get(index)
    }

    fn find_field(&self, field: &Field) -> Option<(usize, bool)> {
        self.inner.find_field(field)
    }

    fn size(&self) -> usize {
        self.inner.size()
    }

    fn set_max_size(&mut self, max_size: usize) {
        self.inner.set_max_size(max_size);
    }
}

pub struct RecvContext<'a> {
    inner: Context<'a>
}

impl<'a> ContextTrait<'a> for RecvContext<'a> {
    fn new(static_table: &'a table::Table) -> Self {
        RecvContext {
            inner: Context::new(static_table)
        }
    }

    fn insert(&mut self, field: Field) {
        self.inner.insert(field)
    }

    fn get(&self, index: usize) -> Option<&Field> {
        self.inner.get(index)
    }

    fn find_field(&self, field: &Field) -> Option<(usize, bool)> {
        self.inner.find_field(field)
    }

    fn size(&self) -> usize {
        self.inner.size()
    }

    fn set_max_size(&mut self, max_size: usize) {
        self.inner.set_max_size(max_size);
    }
}

