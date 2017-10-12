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

extern crate osmium;
#[macro_use] extern crate log;
extern crate pretty_env_logger;

use osmium::http2::{self, net, header, stream as streaming};
use http2::core::ConnectionHandle;
use osmium::shared;

struct MyServer;

#[derive(Debug)]
struct HttpRequest {
    pub headers: header::Headers,
    pub body: Option<String>
}

#[derive(Debug)]
struct HttpResponse {
    pub headers: header::Headers,
    pub body: Option<String>
}

impl From<streaming::StreamRequest> for HttpRequest {
    fn from(stream_request: streaming::StreamRequest) -> HttpRequest {
        HttpRequest {
            headers: stream_request.headers,
            body: stream_request.payload
        }
    }
}

impl From<HttpResponse> for streaming::StreamResponse {
    fn from(http_response: HttpResponse) -> streaming::StreamResponse {
        streaming::StreamResponse {
            informational_headers: Vec::new(),
            headers: http_response.headers,
            payload: http_response.body,
            trailer_headers: None
        }
    }
}

impl shared::server_trait::OsmiumServer for MyServer {
    type Request = HttpRequest;
    type Response = HttpResponse;

    fn process(&self, request: Self::Request, handle: Box<&ConnectionHandle>) -> Self::Response {
        println!("Got request {:?}", request);

        if handle.is_push_enabled() {
            println!("push is enabled!");
        }
        else {
            println!("push is disabled");
        }

        let mut headers = header::Headers::new();
        headers.push(header::HeaderName::PseudoStatus, header::HeaderValue::Num(200));
        headers.push(header::HeaderName::ContentLength, header::HeaderValue::Num(111));
        headers.push(header::HeaderName::ContentType, header::HeaderValue::Str(String::from("text/html")));

        HttpResponse {
            headers: headers,
            body: Some(String::from("<!DOCTYPE html><html><head><title>test</title></head><body><h1>Osmium served me like a beast</h1></body></html>"))
        }
    }
}

fn main() {
    pretty_env_logger::init().unwrap();

    info!("File server example begining");

    let mut settings = shared::server_settings::ServerSettings::default();
    let mut security = shared::server_settings::SecuritySettings::default();
    security.set_ssl_cert_path(String::from("../../../tests/certificate.pfx"));
    settings.set_security(security);

    let handshake = net::https::HttpsH2Handshake::new();
    net::Server::new(MyServer {}, settings).start_server(handshake);
}
