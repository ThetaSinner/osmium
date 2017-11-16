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

// osmium
use http2::settings;
use http2::frame as framing;

pub struct ConnectionSharedState {
    // TODO rename to remote settings
    pub incoming_settings: settings::Settings,
    next_server_created_stream_id: framing::StreamId,
    // If streams were ever made concurrent it would be VITAL that this is locked. It is used to communicate to
    // the client which streams have started processing, or at least the highest numbered one. That means no more
    // streams may start processing once this has been sent.
    highest_started_processing_stream_id: framing::StreamId
}

impl ConnectionSharedState {
    pub fn new() -> Self {
        ConnectionSharedState {
            incoming_settings: settings::Settings::spec_default(),
            next_server_created_stream_id: 2,
            highest_started_processing_stream_id: 0
        }
    }

    pub fn get_next_stream_id_for_locally_initiated_stream(&mut self) -> u32 {
        let id = self.next_server_created_stream_id;
        self.next_server_created_stream_id += 2;
        id
    }

    pub fn notify_processing_started_on_stream(&mut self, stream_id: framing::StreamId) {
        if stream_id > self.highest_started_processing_stream_id {
            self.highest_started_processing_stream_id = stream_id;
        }
    }

    pub fn get_highest_started_processing_stream_id(&self) -> framing::StreamId {
        self.highest_started_processing_stream_id
    }
}
