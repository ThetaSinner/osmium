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
// along with Osmium.  If not, see <http://www.gnu.org/licenses/>.

// std
use std::io;

// tokio
use bytes::BytesMut;
use tokio_io::codec::{Encoder, Decoder};

// osmium
use http::request;
use http::response;

pub struct HttpCodec;

impl Decoder for HttpCodec {
    type Item = request::Request;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> io::Result<Option<Self::Item>> {
        request::from_incoming(buf)
    }
}

impl Encoder for HttpCodec {
    type Item = response::Response;
    type Error = io::Error;

    fn encode(&mut self, item: Self::Item, buf: &mut BytesMut) -> io::Result<()> {
        response::to_outgoing(item, buf);
        Ok(())
    }
}
