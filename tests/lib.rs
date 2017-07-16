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
extern crate curl;

// std
use std::thread;
use std::str;

// curl
use curl::easy::Easy;

// osmium
use osmium::http::{request, response, server, handler, status, header};

#[test]
fn empty_request() {
    pretty_env_logger::init().unwrap();

    thread::spawn(move || {
        debug!("Starting a server");

        struct MyHandler;

        impl handler::Handler for MyHandler {
            fn process(&self, req: request::Request) -> response::Response {
                debug!("Responding to request: {:?}", req);

                let mut headers = header::Headers::new();
                headers.add(header::HeaderName::CustomHeader("Server".to_owned()), header::HeaderValue::Str("Osmium".to_owned()));
                headers.add(header::HeaderName::ContentLength, header::HeaderValue::Num(0));

                response::Response {
                    version: req.version,
                    status: status::HttpStatus::Ok,
                    headers: headers,
                    body: None
                }
            }
        }

        server::run(MyHandler, None);
    });

    let mut response = Vec::new();
    {
        let mut handle = Easy::new();   

        handle.url("http://localhost:8000").unwrap();
        handle.show_header(true).unwrap();
        let mut transfer = handle.transfer();
        transfer.write_function(|new_data| {
            trace!("Reading response line: [{:?}]", new_data);
            response.extend_from_slice(new_data);
            Ok(new_data.len())
        }).unwrap();
        debug!("Making a request to the server");
        transfer.perform().unwrap();
    }

    assert_eq!(response.len(), 54);
    assert_eq!(str::from_utf8(response.as_slice()).unwrap(), "HTTP/1.1 200 OK\r\nContent-Length: 0\r\nServer: Osmium\r\n\r\n");
}
