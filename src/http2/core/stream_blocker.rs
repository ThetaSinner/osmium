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
// along with Osmium. If not, see <http://www.gnu.org/licenses/>.

// std
use std::collections::VecDeque;
use std::collections::HashMap;
use std::collections::hash_map;

// osmium
use http2::frame as framing;

// TODO this never cleans up.
pub struct StreamBlocker {
    blocked_streams: HashMap<framing::StreamId, VecDeque<Box<framing::CompressibleHttpFrame>>>,
    priority: VecDeque<framing::StreamId>
}

// TODO this doesn't have a great interface. These methods have to be called in sequence to some extent.
impl StreamBlocker {
    pub fn new() -> Self {
        StreamBlocker {
            blocked_streams: HashMap::new(),
            priority: VecDeque::new()
        }
    }

    pub fn block_frame(&mut self, stream_id: framing::StreamId, frame: Box<framing::CompressibleHttpFrame>) {
        if self.blocked_streams.contains_key(&stream_id) {
            // This stream is already blocking so append to the queue for this frame
            match self.blocked_streams.entry(stream_id) {
                hash_map::Entry::Occupied(mut entry) => {
                    entry.get_mut().push_front(frame);
                },
                _ => {
                    // TODO there must be a better way to do this. Existence of the entry has been checked.
                    panic!("expected map entry but was not found");
                }
            }
        }
        else {
            // Create a new entry in the blocked streams map.
            let mut q = VecDeque::new();
            q.push_front(frame);
            self.blocked_streams.insert(stream_id, q);

            // Prioritise this stream.
            self.priority.push_front(stream_id);
        }
    }

    pub fn is_blocking(&self, stream_id: framing::StreamId) -> bool {
        self.blocked_streams.contains_key(&stream_id)
    }

    pub fn get_unblock_priorities(&self) -> VecDeque<framing::StreamId> {
        self.priority.clone()
    }

    // Here's a lovely example of a bad interface. You cannot read an entry without the possibility 
    // of modifying it. Therefore forcing this method, which should not change internal state, to
    // require self to be mutable.
    pub fn get_next_send_size(&mut self, stream_id: framing::StreamId) -> Option<i32> {
        match self.blocked_streams.entry(stream_id) {
            hash_map::Entry::Occupied(ref entry) => {
                match entry.get().back() {
                    Some(ref frame) => {
                        match frame.get_frame_type() {
                            framing::FrameType::Data => {
                                Some(frame.get_length())
                            },
                            _ => {
                                Some(0)
                            }
                        }
                    },
                    None => {
                        None
                    }
                }
            },
            _ => {
                None
            }
        }
    }

    pub fn get_next_frame(&mut self, stream_id: framing::StreamId) -> Option<Box<framing::CompressibleHttpFrame>> {
        match self.blocked_streams.entry(stream_id) {
            hash_map::Entry::Occupied(mut queue) => {
                queue.get_mut().pop_back()
            },
            _ => {
                None
            }
        }
    }
}