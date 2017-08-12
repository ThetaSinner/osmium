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

use super::CompressibleHttpFrame;

const SETTINGS_FRAME_TYPE: u8 = 0x4;

const FLAG_ACK: u8 = 0x1;

// std
use std::vec::IntoIter;

// osmium
use http2::settings;

pub struct SettingsParameter {
    name: settings::SettingName,
    value: u32
}

pub struct SettingsFrameCompressModel {
    flags: u8,
    parameters: Vec<SettingsParameter>
}

impl SettingsFrameCompressModel {
    pub fn new() -> Self {
        SettingsFrameCompressModel {
            flags: 0,
            parameters: Vec::new()
        }
    }

    pub fn set_acknowledge(&mut self) {
        self.flags |= FLAG_ACK;
    }

    pub fn add_parameter(&mut self, name: settings::SettingName, value: u32) {
        self.parameters.push(SettingsParameter {
            name: name,
            value: value
        })
    }
}

impl CompressibleHttpFrame for SettingsFrameCompressModel {
    fn get_length(&self) -> i32 {
        6 * self.parameters.len() as i32
    }

    fn get_frame_type(&self) -> u8 {
        SETTINGS_FRAME_TYPE
    }

    fn get_flags(&self) -> u8 {
        self.flags
    }

    fn get_payload(self) -> Vec<u8> {
        let mut result = Vec::new();

        for setting in self.parameters.into_iter() {
            let name = setting.name as u16;
            result.push((name >> 8) as u8);
            result.push(name as u8);

            let value = setting.value;
            result.push((value >> 24) as u8);
            result.push((value >> 16) as u8);
            result.push((value >> 8) as u8);
            result.push(value as u8);
        }

        result
    }
}

pub struct SettingsFrame {
    acknowledge: bool,
    parameters: Vec<SettingsParameter>
}

impl SettingsFrame {
    pub fn new(frame_header: &super::FrameHeader, frame: &mut IntoIter<u8>) -> Self {
        if frame_header.flags & FLAG_ACK == FLAG_ACK {
            // TODO handle error
            assert_eq!(0, frame_header.length);

            SettingsFrame {
                acknowledge: true,
                parameters: Vec::new()
            }
        }
        else {
            // TODO handle error
            // Each settings parameter is 6 octets, so the payload length
            // should be a multiple of 6.
            assert_eq!(0, frame_header.length % 6);
            
            let mut settings_frame = SettingsFrame {
                acknowledge: false,
                parameters: Vec::new()
            };

            for _ in 0..(frame_header.length / 6) {
                let raw_name = 
                    (frame.next().unwrap() as u16) << 8 +
                    (frame.next().unwrap() as u16);

                let opt_name = settings::to_setting_name(
                    raw_name
                );

                if opt_name.is_none() {
                    info!("Unknown setting {:?} on settings frame with header {:?}", raw_name, frame_header);
                }
                else {
                    settings_frame.parameters.push(SettingsParameter {
                        name: opt_name.unwrap(),
                        value:
                            (frame.next().unwrap() as u32) << 24 +
                            (frame.next().unwrap() as u32) << 16 +
                            (frame.next().unwrap() as u32) << 8 +
                            frame.next().unwrap() as u32
                    });
                }
            }

            settings_frame
        }
    }

    pub fn is_acknowledge(&self) -> bool {
        self.acknowledge   
    }

    pub fn get_parameters(&self) -> &[SettingsParameter] {
        self.parameters.as_slice()
    }
}
