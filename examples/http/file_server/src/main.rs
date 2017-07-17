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

use osmium::http::{handler, server, request, response, header, status};

struct FileServerHandler;

impl handler::Handler for FileServerHandler {
    fn process(&self, req: request::Request) -> response::Response {
        debug!("Got request: {:?}", req);

        let mut headers = header::Headers::new();
        headers.add(header::HeaderName::CustomHeader("Server".to_owned()), header::HeaderValue::Str("Osmium".to_owned()));

        match req.uri.as_ref() {
            "/index.html" => {
                headers.add(header::HeaderName::ContentLength, header::HeaderValue::Num(95));
                headers.add(header::HeaderName::CustomHeader("Content-Type".to_owned()), header::HeaderValue::Str("text/html".to_owned()));

                response::Response {
                    version: req.version,
                    status: status::HttpStatus::Ok,
                    headers: headers,
                    body: Some("<!DOCTYPE html><html><head><title>osmium</title></head><body><h1>hello world</h1></body></html>".to_owned())
                }
            },
            _ => {
                headers.add(header::HeaderName::ContentLength, header::HeaderValue::Num(0));

                response::Response {
                    version: req.version,
                    status: status::HttpStatus::NotFound,
                    headers: headers,
                    body: None
                }
            }
        }
    }
}

fn main() {
    pretty_env_logger::init().unwrap();

    info!("File server example begining");

    let settings = server::Settings {
        host: Some("0.0.0.0".to_owned()),
        port: Some(8000)
    };

    info!("Starting server...");
    info!("Visit localhost:8000/index.html");

    server::run(FileServerHandler, Some(settings));
}
