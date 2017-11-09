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
use std::collections::VecDeque;

#[derive(Clone, Debug)]
pub struct Field {
    pub name: String,
    pub value: String
}

// TODO the hpack spec sugggests storing strings, then using pointers to those strings and counting references,
// rather than storing duplicate strings. The strings could then be stored in a hash, which would improve
// the find_field lookup.
// TODO because table entries may be evicted when a new field is added, care must be taken to ensure that fields which
// refer to their name via a pointer remain valid when values are removed. Note that such a field will not be in the table
// and will hence not be included in the reference count (and shouldn't be as this could leave the table with excess data
// required for storage if it fails to be added). See the TODO above.
pub struct Table {
    fields: VecDeque<Field>,
    /// Number of octets in `fields`
    size: usize,
    /// Maximmum number of octets to store in 'fields'
    max_size: usize,
    /// The maximum size that max_size may be set to
    // TOOD this is not respected
    max_size_setting: usize
}

impl Table {
    pub fn new() -> Self {
        Table {
            fields: VecDeque::new(),
            size: 0,
            max_size: 51200, // for now allocate the hardcoded max allowed size setting
            max_size_setting: 512000 // 64kB
        }
    }

    pub fn push_front(&mut self, field: Field) {
        // TODO this is mixing bit length and byte length, need the number of bits used in the strings.
        let storage_size_required = field.name.len() + field.value.len() + 32;

        // Evict entries from the table until the new entry can be inserted without exceeding
        // self.max_size
        let max_actual_size = self.max_size - storage_size_required;
        self.ensure_max_size(max_actual_size);

        // The hpack spec states that such a value should cause the table to be emptied, but storing it would be an error.
        if storage_size_required > self.max_size {
            // TODO error.
            panic!("Will not store field which exceeds table size");
        }

        // Add the field to the table.
        self.fields.push_front(field);

        // Update the actual size of the table.
        // For this to be correct name and value must not be Huffman coded
        self.size += storage_size_required;
    }

    // TODO can I make the compiler understand this is only to be used in one place
    // without making a huge mess?

    /// Append an element to the bottom of the table. This is only to be used
    /// when building the static table. It bypasses the size checks and allows
    /// the static table to be built in order
    pub fn push_back(&mut self, field: Field) {
        self.fields.push_back(field);
    }

    pub fn get(&self, index: usize) -> Option<&Field> {
        if index < self.fields.len() {
            Some(&self.fields[index])
        }
        else {
            None
        }
    }

    pub fn len(&self) -> usize {
        self.fields.len()
    }

    pub fn get_size(&self) -> usize {
        self.size
    }

    // returns optional, if some then a tuple. The first item is the index, the second item 
    // is a boolean indicating whether the input field matched the returned index with name and value (true)
    // or just name (false)
    pub fn find_field(&self, field: &Field) -> Option<(usize, bool)> {
        let matched_partition: (Vec<(usize, bool)>, Vec<(usize, bool)>) = self.fields.iter().enumerate().filter_map(|indexed_field| {
            if (&indexed_field.1.name).eq_ignore_ascii_case(&field.name) {
                trace!("found match for {} at index {}", field.name, indexed_field.0);
                Some((indexed_field.0, indexed_field.1.value == field.value))
            }
            else {
                None
            }
        }).partition(|x| {
            x.1
        });

        // Note that the indices we've found start from 0, but the table is numbered from 1.
        // so we add 1 to the index before returning it.

        if matched_partition.0.is_empty() {
            if matched_partition.1.is_empty() {
                None
            }
            else {
                let result = matched_partition.1[0];
                Some((result.0 + 1, result.1))
            }
        }
        else {
            let result = matched_partition.0[0];
            Some((result.0 + 1, result.1))
        }
    }

    pub fn set_max_size(&mut self, max_size: usize) {
        // TODO handle without crash. Things like handling this error is likely to be in the http2 spec
        // and is not included in the hpack spec as far as I know.
        if max_size > self.max_size_setting {
            panic!("May not set table size to greater than {}", self.max_size_setting);
        }

        self.max_size = max_size;
        self.ensure_max_size(max_size)
    }

    fn ensure_max_size(&mut self, max_size: usize) {
        // Note that usize is non-negative, so a zero check is not required to guarantee termination
        while self.size > max_size {
            // If the list is empty and the unwrap panics then self.size has been incorrectly maintained or corrupted
            // TOOD error
            let popped = self.fields.pop_back().unwrap();

            self.size -= popped.name.len() + popped.value.len() + 32;
        }
    }
}
