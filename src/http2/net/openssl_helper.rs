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

use openssl::pkcs12::Pkcs12;
use openssl::ssl::{SslMethod, SslAcceptor, SslAcceptorBuilder};
use std::fs::File;
use std::io::{Read, Write};
use std::io;
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::thread;
use futures::Stream;

use std::sync::mpsc;
use bytes::BufMut;
use futures::Poll;
use futures::Future;
use futures::sync::mpsc as futures_mpsc;
use tokio_io::{AsyncRead, AsyncWrite};
use tokio_openssl::{SslStream, SslAcceptorExt};
use shared::server_settings::SecuritySettings;

const ALPN_PROTOCOLS: &[&[u8]] = &[&[0x68, 0x32]];

pub fn make_acceptor(security_settings: &SecuritySettings) -> SslAcceptor
{
    // TODO shouldn't load the same certificate over and over again when new connections are made.

    // In this example we retrieve our keypair and certificate chain from a PKCS #12 archive,
    // but but they can also be retrieved from, for example, individual PEM- or DER-formatted
    // files. See the documentation for the `PKey` and `X509` types for more details.
    let mut file = File::open(security_settings.get_ssl_cert_path()).expect("ssl certificate not found");
    let mut pkcs12 = vec![];
    file.read_to_end(&mut pkcs12).unwrap();
    let pkcs12 = Pkcs12::from_der(&pkcs12).unwrap();
    let identity = pkcs12.parse(security_settings.get_ssl_cert_pass()).unwrap(); // this is set when you create the password with openssl

    let mut acceptor_builder = SslAcceptorBuilder::mozilla_intermediate(SslMethod::tls(),
                                                            &identity.pkey,
                                                            &identity.cert,
                                                            &identity.chain).unwrap();

    {
        let context_builder = acceptor_builder.builder_mut();

        // This completely disables certificate verification, because the certificates I'm using are 
        // self signed. PLEASE REMOVE THIS BEFORE DEPLOYING ANYWHERE!
        // TODO and now it doesn't need it!?
        // context_builder.set_verify(openssl::ssl::SslVerifyMode::empty());

        // Require http2 protocol in alpn negotiation.
        context_builder.set_alpn_protocols(ALPN_PROTOCOLS).unwrap();
    }

    acceptor_builder.build()
}
