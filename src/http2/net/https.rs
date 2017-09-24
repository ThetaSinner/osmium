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
use http2::frame as framing;

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
    fn attempt_handshake<S>(&self, stream: S, settings_response: Box<framing::settings::SettingsFrameCompressModel>) -> Box<Future<Item = future::FutureResult<HandshakeCompletion<SslStream<S>>, HandshakeError<SslStream<S>>>, Error = io::Error>>
        where S: AsyncRead + AsyncWrite + 'static
    {
        let acceptor = openssl_helper::make_acceptor();

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
                if buf == PREFACE {
                    let header_buf = [0; 9];
                    let handshake_settings_future: Box<Future<Item = future::FutureResult<HandshakeCompletion<SslStream<S>>, HandshakeError<SslStream<S>>>, Error = io::Error>> = 
                    Box::new(
                        tokio_io::io::read_exact(stream, header_buf)
                        .and_then(|(stream, buf)| {
                            let frame_header = framing::decompress_frame_header(buf.to_vec());

                            let mut payload_buf = Vec::with_capacity(frame_header.length as usize);
                            payload_buf.resize(frame_header.length as usize, 0);
                            future::ok(frame_header)
                            .join(
                                tokio_io::io::read_exact(stream, payload_buf)
                            )
                        })
                        .and_then(move |(frame_header, (stream, buf))| {
                            // TODO need to check frame type and stream id
                            let settings_frame = framing::settings::SettingsFrame::new(&frame_header, &mut buf.to_vec().into_iter());

                            let mut response = PREFACE.to_vec();
                            response.extend(framing::compress_frame(settings_response, 0x0));
                            future::ok(settings_frame)
                            .join(
                                tokio_io::io::write_all(stream, response)
                            )
                        })
                        .map(|(settings_frame, (stream, _))| {
                            future::ok(HandshakeCompletion { stream, settings_frame })
                        })
                    );

                    handshake_settings_future
                }
                else {
                    Box::new(
                        future::ok(future::err(HandshakeError::DidNotUpgrade(stream, buf.to_vec())))
                    )
                }
            })
        )
    }
}
