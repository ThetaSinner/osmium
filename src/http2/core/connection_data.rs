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

// TODO rename this
pub struct ConnectionData {
    pub incoming_settings: settings::Settings,
    next_server_created_stream_id: framing::StreamId
}

impl ConnectionData {
    pub fn new() -> Self {
        ConnectionData {
            incoming_settings: settings::Settings::spec_default(),
            next_server_created_stream_id: 2
        }
    }

    pub fn get_next_stream_id_for_locally_initiated_stream(&mut self) -> u32 {
        let id = self.next_server_created_stream_id;
        self.next_server_created_stream_id += 2;
        id
    }
}
