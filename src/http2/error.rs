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

pub enum HttpError {
    ConnectionError(ErrorCode, ErrorName),
    StreamError(ErrorCode, ErrorName)
}

pub enum ErrorCode {
    // The associated condition is not a result of an error. For example, a GOAWAY might include this code to indicate graceful shutdown of a connection.
    NoError,
    // The endpoint detected an unspecific protocol error. This error is for use when a more specific error code is not available.
    ProtocolError,
    // The endpoint encountered an unexpected internal error.
    InternalError,
    // The endpoint detected that its peer violated the flow-control protocol.
    FlowControlError,
    // The endpoint sent a SETTINGS frame but did not receive a response in a timely manner. See Section 6.5.3 ("Settings Synchronization").
    SettingsTimeout,
    // The endpoint received a frame after a stream was half-closed.
    StreamClosed,
    // The endpoint received a frame with an invalid size.
    FrameSizeError,
    // The endpoint refused the stream prior to performing any application processing (see Section 8.1.4 for details).
    RefusedStream,
    // Used by the endpoint to indicate that the stream is no longer needed.
    Cancel,
    // The endpoint is unable to maintain the header compression context for the connection.
    CompressionError,
    // The connection established in response to a CONNECT request (Section 8.3) was reset or abnormally closed.
    ConnectError,
    // The endpoint detected that its peer is exhibiting a behavior that might be generating excessive load.
    EnhanceYourCalm,
    // The underlying transport has properties that do not meet minimum security requirements (see Section 9.2).
    InadequateSecurity,
    // The endpoint requires that HTTP/1.1 be used instead of HTTP/2.
    Http11Required
}

impl From<ErrorCode> for u32 {
    fn from(error_code: ErrorCode) -> u32 {
        match error_code {
            ErrorCode::NoError => 0x0,
            ErrorCode::ProtocolError => 0x1,
            ErrorCode::InternalError => 0x2,
            ErrorCode::FlowControlError => 0x3,
            ErrorCode::SettingsTimeout => 0x4,
            ErrorCode::StreamClosed => 0x5,
            ErrorCode::FrameSizeError => 0x6,
            ErrorCode::RefusedStream => 0x7,
            ErrorCode::Cancel => 0x8,
            ErrorCode::CompressionError => 0x9,
            ErrorCode::ConnectError => 0xa,
            ErrorCode::EnhanceYourCalm => 0xb,
            ErrorCode::InadequateSecurity => 0xc,
            ErrorCode::Http11Required => 0xd
        }
    }
}

pub fn to_error_code(error_code: u32) -> Option<ErrorCode> {
    match error_code {
        0x0 => Some(ErrorCode::NoError),
        0x1 => Some(ErrorCode::ProtocolError),
        0x2 => Some(ErrorCode::InternalError),
        0x3 => Some(ErrorCode::FlowControlError),
        0x4 => Some(ErrorCode::SettingsTimeout),
        0x5 => Some(ErrorCode::StreamClosed),
        0x6 => Some(ErrorCode::FrameSizeError),
        0x7 => Some(ErrorCode::RefusedStream),
        0x8 => Some(ErrorCode::Cancel),
        0x9 => Some(ErrorCode::CompressionError),
        0xa => Some(ErrorCode::ConnectError),
        0xb => Some(ErrorCode::EnhanceYourCalm),
        0xc => Some(ErrorCode::InadequateSecurity),
        0xd => Some(ErrorCode::Http11Required),
        _ => None
    }
}

pub enum ErrorName {
    StreamIdentifierOnConnectionFrame
}

impl From<ErrorName> for Vec<u8> {
    fn from(error_name: ErrorName) -> Vec<u8> {
        match error_name {
            ErrorName::StreamIdentifierOnConnectionFrame => {
                "unexpected stream identifier on connection frame"
            }
        }.to_owned().as_bytes().to_vec()
    }
}
