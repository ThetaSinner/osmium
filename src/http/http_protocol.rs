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
use tokio_io::codec::Framed;
use tokio_io::{AsyncRead, AsyncWrite};
use tokio_proto::pipeline::ServerProto;

// osmium
use http::request;
use http::response;
use http::http_codec;

pub struct HttpProtocol;

impl<T: AsyncRead + AsyncWrite + 'static> ServerProto<T> for HttpProtocol {
    type Request = request::Request;
    type Response = response::Response;
    type Transport = Framed<T, http_codec::HttpCodec>;
    type BindTransport = io::Result<Framed<T, http_codec::HttpCodec>>;

    fn bind_transport(&self, io: T) -> io::Result<Framed<T, http_codec::HttpCodec>> {
        Ok(io.framed(http_codec::HttpCodec))
    }
}
