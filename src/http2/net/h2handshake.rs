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

use futures::future::{self, Future};
use tokio_io::{AsyncRead, AsyncWrite};
use std::io;
use tokio_openssl::SslStream;
use http2::frame as framing;

pub trait H2Handshake {
    fn attempt_handshake<S>(&self, stream: S, settings_response: Box<framing::settings::SettingsFrameCompressModel>) -> Box<Future<Item = future::FutureResult<HandshakeCompletion<SslStream<S>>, HandshakeError<SslStream<S>>>, Error = io::Error>>
        where S: AsyncRead + AsyncWrite + 'static;
}

#[derive(Debug)]
pub struct HandshakeCompletion<S>
    where S: AsyncRead + AsyncWrite
{
    pub stream: S,
    pub settings_frame: framing::settings::SettingsFrame
}

#[derive(Debug)]
pub enum HandshakeError<S>
    where S: AsyncRead + AsyncWrite
{
    DidNotUpgrade(S, Vec<u8>)
}
