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
use tokio_core::net as tokio_net;
use std::io;
use tokio_openssl::SslStream;
use http2::frame as framing;

// Sadly rust doesn't completely support generic traits. To keep the code simpler this is naming the transport directly.
// It would be nice if the code at this level wasn't tied to the underlying transport but the traits needed are in tokio_io
// anyway so...

pub trait H2Handshake {
    fn attempt_handshake(&self, stream: tokio_net::TcpStream, settings_response: Box<framing::settings::SettingsFrameCompressModel>) -> Box<Future<Item = future::FutureResult<HandshakeCompletion, HandshakeError>, Error = io::Error>>;
}

#[derive(Debug)]
pub struct HandshakeCompletion
{
    pub stream: SslStream<tokio_net::TcpStream>,
    pub settings_frame: framing::settings::SettingsFrame
}

#[derive(Debug)]
pub enum HandshakeError
{
    DidNotUpgrade(SslStream<tokio_net::TcpStream>, Vec<u8>)
}
