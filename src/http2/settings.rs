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
use http2::error;

pub const INITIAL_MAX_FRAME_SIZE: u32 = 0x4000;
pub const MAXIMUM_MAX_FRAME_SIZE: u32 = 0xFFFFFF;

pub const INITIAL_FLOW_CONTROL_WINDOW_SIZE: u32 = 0xFFFF;
pub const MAXIMUM_FLOW_CONTROL_WINDOW_SIZE: u32 = 0x7FFFFFFF;

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct Settings {
    pub header_table_size: u32,
    pub enable_push: bool,
    pub max_concurrent_streams: Option<u32>,
    pub initial_window_size: u32,
    pub max_frame_size: u32,
    pub max_header_list_size: Option<u32>
}

impl SettingsParameter {
    pub fn new(name: SettingName, value: u32) -> Self {
        SettingsParameter { name, value }
    }

    pub fn get_name(&self) -> SettingName {
        self.name.clone()
    }

    pub fn get_value(&self) -> u32 {
        self.value
    }
}

impl Settings {
    pub fn spec_default() -> Self {
        Settings {
            header_table_size: 4096,
            enable_push: true,
            max_concurrent_streams: None,
            initial_window_size: INITIAL_FLOW_CONTROL_WINDOW_SIZE,
            max_frame_size: INITIAL_MAX_FRAME_SIZE,
            max_header_list_size: None
        }
    }

    pub fn apply_changes(&mut self, changes: &[SettingsParameter]) -> Result<Vec<SettingName>, error::HttpError> {
        let mut changes_applied = Vec::with_capacity(changes.len());

        for setting in changes {
            match setting.get_name() {
                SettingName::SettingsHeaderTableSize => {
                    self.header_table_size = setting.get_value();

                    changes_applied.push(setting.get_name());
                },
                SettingName::SettingsEnablePush => {
                    match setting.get_value() {
                        0 => {
                            self.enable_push = false;
                        },
                        1 => {
                            self.enable_push = true;
                        },
                        _ => {
                            // (6.5.2) Any value other than 0 or 1 MUST be treated as a connection 
                            // error (Section 5.4.1) of type PROTOCOL_ERROR.
                            return Err(
                                error::HttpError::ConnectionError(
                                    error::ErrorCode::ProtocolError,
                                    error::ErrorName::EnablePushSettingInvalidValue
                                )
                            );
                        }
                    }

                    changes_applied.push(setting.get_name())
                },
                SettingName::SettingsMaxConcurrentStreams => {
                    self.max_concurrent_streams = Some(setting.get_value());

                    changes_applied.push(setting.get_name());
                },
                SettingName::SettingsInitialWindowSize => {
                    let val = setting.get_value();

                    if val <= MAXIMUM_FLOW_CONTROL_WINDOW_SIZE {
                        self.initial_window_size = val;
                    }
                    else {
                        // (6.5.2) Values above the maximum flow-control window size of 231-1 MUST be treated as a 
                        // connection error (Section 5.4.1) of type FLOW_CONTROL_ERROR.
                        return Err(
                            error::HttpError::ConnectionError(
                                error::ErrorCode::ProtocolError,
                                error::ErrorName::InvalidInitialWindowSize
                            )
                        )
                    }

                    changes_applied.push(setting.get_name());
                },
                SettingName::SettingsMaxFrameSize => {
                    let val = setting.get_value();

                    if INITIAL_MAX_FRAME_SIZE <= val && val <= MAXIMUM_MAX_FRAME_SIZE {
                        self.max_frame_size = val;
                    }
                    else {
                        // (6.5.2) The initial value is 214 (16,384) octets. The value advertised by an endpoint MUST be between this initial 
                        // value and the maximum allowed frame size (224-1 or 16,777,215 octets), inclusive. Values outside this range MUST 
                        // be treated as a connection error (Section 5.4.1) of type PROTOCOL_ERROR.
                        return Err(
                            error::HttpError::ConnectionError(
                                error::ErrorCode::ProtocolError,
                                error::ErrorName::InvalidMaxFrameSize
                            )
                        );
                    }

                    changes_applied.push(setting.get_name());
                },
                SettingName::SettingsMaxHeaderListSize => {
                    self.max_header_list_size = Some(setting.get_value());

                    changes_applied.push(setting.get_name());
                }
            }
        }

        Ok(changes_applied)
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
