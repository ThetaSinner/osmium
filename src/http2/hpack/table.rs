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
    max_size: usize
}

impl Table {
    pub fn push_front(&mut self, field: Field) {
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

    // returns optional, if some then a tuple. The first item is the index, the second item 
    // is a boolean indicating whether the input field matched the returned index with name and value (true)
    // or just name (false)
    pub fn find_field(&self, field: &Field) -> Option<(usize, bool)> {
        let matched_partition: (Vec<(usize, bool)>, Vec<(usize, bool)>) = self.fields.iter().enumerate().filter_map(|indexed_field| {
            if indexed_field.1.name == field.name {
                Some((indexed_field.0, indexed_field.1.value == field.value))
            }
            else {
                None
            }
        }).partition(|x| {
            x.1
        });

        if matched_partition.0.is_empty() {
            if matched_partition.1.is_empty() {
                None
            }
            else {
                Some(matched_partition.1[0])
            }
        }
        else {
            Some(matched_partition.0[0])
        }
    }

    pub fn set_max_size(&mut self, max_size: usize) {
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
