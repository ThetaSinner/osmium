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

use http2::frame::window_update;
use http2::error;
use http2::settings;

pub fn check_conn_window_update(
    decoded_frame: Result<window_update::WindowUpdateFrame, error::HttpError>,
    send_window: u32
) -> Result<window_update::WindowUpdateFrame, error::HttpError>
{
    match decoded_frame {
        Ok(frame) => {
            if frame.get_window_size_increment() == 0 {
                Err(error::HttpError::ConnectionError(
                    error::ErrorCode::ProtocolError,
                    error::ErrorName::ZeroWindowSizeIncrement
                ))
            }
            else if settings::MAXIMUM_FLOW_CONTROL_WINDOW_SIZE - frame.get_window_size_increment() < send_window {
                Err(error::HttpError::ConnectionError(
                    error::ErrorCode::FlowControlError,
                    error::ErrorName::WindowUpdateWouldCauseSendWindowToExceedLimit
                ))
            }
            else {
                Ok(frame)
            }
        },
        Err(e) => Err(e)
    }
}

pub fn check_stream_window_update(
    decoded_frame: Result<window_update::WindowUpdateFrame, error::HttpError>,
    send_window: u32
) -> Result<window_update::WindowUpdateFrame, error::HttpError>
{
    match decoded_frame {
        Ok(frame) => {
            // (6.9) A receiver MUST treat the receipt of a WINDOW_UPDATE frame with an flow-control 
            // window increment of 0 as a stream error (Section 5.4.2) of type PROTOCOL_ERROR
            if frame.get_window_size_increment() == 0 {
                Err(error::HttpError::StreamError(
                    error::ErrorCode::ProtocolError,
                    error::ErrorName::ZeroWindowSizeIncrement
                ))
            }
            else if settings::MAXIMUM_FLOW_CONTROL_WINDOW_SIZE - frame.get_window_size_increment() < send_window {
                Err(error::HttpError::StreamError(
                    error::ErrorCode::FlowControlError,
                    error::ErrorName::WindowUpdateWouldCauseSendWindowToExceedLimit
                ))
            }
            else {
                Ok(frame)
            }
        },
        Err(e) => Err(e)
    }
}
