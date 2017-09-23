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

use super::h2handshake::{self, HandshakeCompletion, HandshakeError};
use super::openssl_helper;

use futures::future::{self, Future};
use tokio_io::{AsyncRead, AsyncWrite};
use std::io;
use tokio_io;
use tokio_openssl::{SslAcceptorExt, SslStream};

const PREFACE: [u8; 24] = [0x50, 0x52, 0x49, 0x20, 0x2a, 0x20, 0x48, 0x54, 0x54, 0x50, 0x2f, 0x32, 0x2e, 0x30, 0x0d, 0x0a, 0x0d, 0x0a, 0x53, 0x4d, 0x0d, 0x0a, 0x0d, 0x0a];

pub struct HttpsH2Handshake {
}

impl HttpsH2Handshake {
    pub fn new() -> Self {
        HttpsH2Handshake {
        }
    }
}

impl h2handshake::H2Handshake for HttpsH2Handshake {
    fn attempt_handshake<S>(&self, stream: S) -> Box<Future<Item = future::FutureResult<HandshakeCompletion<SslStream<S>>, HandshakeError<SslStream<S>>>, Error = io::Error>>
        where S: AsyncRead + AsyncWrite + 'static
    {
        let acceptor = openssl_helper::make_acceptor();

        // TODO still need to receive settings frame after connection preface. and send our own settings.

        Box::new(
            acceptor.accept_async(stream)
            .map_err(|e| {
                println!("accept error {:?}", e);
                io::Error::new(io::ErrorKind::Other, e)
            })
            .and_then(|stream| {
                let buf = [0; 24];
                tokio_io::io::read_exact(stream, buf)
            })
            .and_then(|(stream, buf)| {
                // println!("received {:?}", String::from_utf8(buf.to_vec()).unwrap());

                if buf == PREFACE {
                    tokio_io::io::write_all(stream, PREFACE.to_vec()).join(Ok(Vec::with_capacity(0)))
                }
                else {
                    tokio_io::io::write_all(stream, Vec::with_capacity(0)).join(Ok(buf.to_vec()))
                }
            })
            .map(|((stream, _), buf)| {
                if buf.len() == 0 {
                    future::ok(HandshakeCompletion {
                        stream: stream
                    })
                }
                else {
                    future::err(HandshakeError::DidNotUpgrade(stream, buf))
                }
            })
        )
    }
}
