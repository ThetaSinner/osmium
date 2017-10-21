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

// In order to enforce any resrictions using flow control, it would be necessary to have some concept of load.
// For example, tracking the size of payloads being sent to active streams would give some measure of load.

pub fn get_window_update_amount(
    current_window_size: u32
) -> u32 {
    // Top up the connection flow control window to the maximum size.
    settings::MAXIMUM_FLOW_CONTROL_WINDOW_SIZE - current_window_size
}
