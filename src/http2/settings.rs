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

#[derive(Debug)]
pub enum SettingName {
    SettingsHeaderTableSize,
    SettingsEnablePush,
    SettingsMaxConcurrentStreams,
    SettingsInitialWindowSize,
    SettingsMaxFrameSize,
    SettingsMaxHeaderListSize
}

#[derive(Debug)]
pub struct SettingsParameter {
    name: SettingName,
    value: u32
}

#[derive(Debug)]
pub struct Settings {
    pub header_table_size: u32,
    pub enable_push: bool,
    pub max_concurrent_streams: u32,
    pub initial_window_size: u32,
    pub max_frame_size: u32,
    pub max_header_list_size: Option<u32>
}

impl Settings {
    pub fn spec_default() -> Self {
        Settings {
            header_table_size: 4096,
            enable_push: true,
            max_concurrent_streams: 100,
            initial_window_size: 65535,
            max_frame_size: 16384,
            max_header_list_size: None
        }
    }
}

impl From<SettingName> for u16 {
    fn from(name: SettingName) -> u16 {
        match name {
            SettingName::SettingsHeaderTableSize => 0x1,
            SettingName::SettingsEnablePush => 0x2,
            SettingName::SettingsMaxConcurrentStreams => 0x3,
            SettingName::SettingsInitialWindowSize => 0x4,
            SettingName::SettingsMaxFrameSize => 0x5,
            SettingName::SettingsMaxHeaderListSize => 0x6
        }
    }
}

pub fn to_setting_name(name: u16) -> Option<SettingName> {
    match name {
        0x1 => Some(SettingName::SettingsHeaderTableSize),
        0x2 => Some(SettingName::SettingsEnablePush),
        0x3 => Some(SettingName::SettingsMaxConcurrentStreams),
        0x4 => Some(SettingName::SettingsInitialWindowSize),
        0x5 => Some(SettingName::SettingsMaxFrameSize),
        0x6 => Some(SettingName::SettingsMaxHeaderListSize),
        _ => None
    }
}
