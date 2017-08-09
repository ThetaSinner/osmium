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
use http2::frame::FrameHeader;
use http2::error::ErrorCode;

pub enum StreamState {
    Idle,
    ReservedLocal,
    ReservedRemote,
    Open,
    HalfClosedLocal,
    HalfClosedRemote,
    Closed
}

pub fn next_state(current_state: &StreamState, header: &FrameHeader) -> Result<StreamState, ErrorCode> {
    match current_state {
        &StreamState::Idle => {
            // TODO
        },
        _ => {
            // TODO
        }
    }

    Ok(StreamState::Idle)
}
