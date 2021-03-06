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
use std::vec::IntoIter;

// osmium
use http2::settings;
use super::CompressibleHttpFrame;
use super::FrameType;
use http2::error;

const FLAG_ACK: u8 = 0x1;

// TODO this has been copied out to the http2::settings module.
#[derive(Debug, Clone)]
pub struct SettingsParameter {
    name: settings::SettingName,
    value: u32
}

impl SettingsParameter {
    pub fn get_name(&self) -> &settings::SettingName {
        &self.name
    }

    pub fn get_value(&self) -> u32 {
        self.value
    }
}

#[derive(Debug, Clone)]
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

    fn get_frame_type(&self) -> FrameType {
        FrameType::Settings
    }

    fn get_flags(&self) -> u8 {
        self.flags
    }

    fn get_payload(self: Box<Self>) -> Vec<u8> {
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

#[derive(Debug)]
pub struct SettingsFrame {
    acknowledge: bool,
    parameters: Vec<SettingsParameter>
}

// Note that where new is called on vector below, no allocation of the underlying array takes place until an attempt is made to
// write to the vector. This means that new_noop and new_acknowledge which don't use parameters, never allocate an array for them.
// The user of this structure can't also can't get this wrong because there is no way to write to parameters from outside this structure.
impl SettingsFrame {
    pub fn new_noop() -> Self {
        SettingsFrame {
            acknowledge: false,
            parameters: Vec::new()
        }
    }

    fn new_acknowledge() -> Self {
        SettingsFrame {
            acknowledge: true,
            parameters: Vec::new()
        }
    }

    fn new_settings(number_of_settings_in_payload: u32) -> Self {
        SettingsFrame {
            acknowledge: false,
            parameters: Vec::with_capacity(number_of_settings_in_payload as usize)
        }
    }

    pub fn new(frame_header: &super::FrameHeader, frame: &mut IntoIter<u8>) -> Result<Self, error::HttpError> {
        // Handle decoding a settings payload with the acknowledge flag set.
        if frame_header.flags & FLAG_ACK == FLAG_ACK {
            let result = if frame_header.length != 0 {
                Err(error::HttpError::ConnectionError(
                    error::ErrorCode::FrameSizeError,
                    error::ErrorName::SettingsAcknowledgementWithNonZeroPayloadLength
                ))
            }
            else {
                Ok(SettingsFrame::new_acknowledge())
            };

            return result;
        }

        // Each settings parameter is 6 octets, so the payload length should be a multiple of 6.
        if frame_header.length % 6 != 0 {
            return Err(error::HttpError::ConnectionError(
                error::ErrorCode::FrameSizeError,
                error::ErrorName::SettingsFramePayloadSizeNotAMultipleOfSix
            ));
        }

        let number_of_settings_in_payload = frame_header.length / 6;
        let mut settings_frame = SettingsFrame::new_settings(number_of_settings_in_payload);

        for _ in 0..number_of_settings_in_payload {
            let raw_name = 
                ((frame.next().unwrap() as u16) << 8) +
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
                        ((frame.next().unwrap() as u32) << 24) +
                        ((frame.next().unwrap() as u32) << 16) +
                        ((frame.next().unwrap() as u32) << 8) +
                        frame.next().unwrap() as u32
                });
            }
        }

        Ok(settings_frame)
    }

    pub fn is_acknowledge(&self) -> bool {
        self.acknowledge   
    }

    pub fn get_parameters(&self) -> &[SettingsParameter] {
        self.parameters.as_slice()
    }
}

#[test]
fn example_payload() {
    let header = super::FrameHeader {
        length: 18,
        frame_type: Some(super::FrameType::Settings),
        flags: 0,
        stream_id: 0
    };
    
    // This is the settings payload the curl sends on with an http2 upgrade request.
    let payload = vec![0, 3, 0, 0, 0, 100, 0, 4, 64, 0, 0, 0, 0, 2, 0, 0, 0, 0];

    let decoded = SettingsFrame::new(&header, &mut payload.into_iter());

    // TODO assert.
    println!("{:?}", decoded);
}
