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

        let settings = server::Settings {
            host: Some("0.0.0.0".to_owned()),
            port: Some(8001)
        };

        server::run(MyHandler, Some(settings));
    });

    let mut response = Vec::new();
    {
        let mut handle = Easy::new();   

        handle.url("http://localhost:8001").unwrap();
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

    let response_text = String::from_utf8(response).unwrap();
    assert!(response_text.starts_with("HTTP/1.1 200 OK\r\n"));
    assert!(response_text.contains("\r\nServer: Osmium"));
    assert!(response_text.contains("\r\nContent-Length: 0"));
    assert!(response_text.ends_with("\r\n\r\n"));
}

#[test]
fn serve_file() {
    thread::spawn(move || {
        debug!("Starting a server");

        struct MyHandler;

        impl handler::Handler for MyHandler {
            fn process(&self, req: request::Request) -> response::Response {
                debug!("Responding to request: {:?}", req);

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

        let settings = server::Settings {
            host: Some("0.0.0.0".to_owned()),
            port: Some(8002)
        };

        server::run(MyHandler, Some(settings));
    });

    let mut response = Vec::new();
    {
        let mut handle = Easy::new();   

        handle.url("http://localhost:8002/index.html").unwrap();
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

    assert_eq!(response.len(), 175);

    let response_text = String::from_utf8(response).unwrap();
    assert!(response_text.starts_with("HTTP/1.1 200 OK\r\n"));
    assert!(response_text.contains("\r\nServer: Osmium"));
    assert!(response_text.contains("\r\nContent-Length: 95"));
    assert!(response_text.contains("\r\nContent-Type: text/html"));
    assert!(response_text.ends_with("\r\n\r\n<!DOCTYPE html><html><head><title>osmium</title></head><body><h1>hello world</h1></body></html>"));    
}

#[test]
fn serve_file_not_found() {
    thread::spawn(move || {
        debug!("Starting a server");

        struct MyHandler;

        impl handler::Handler for MyHandler {
            fn process(&self, req: request::Request) -> response::Response {
                debug!("Responding to request: {:?}", req);

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

        let settings = server::Settings {
            host: Some("0.0.0.0".to_owned()),
            port: Some(8003)
        };

        server::run(MyHandler, Some(settings));
    });

    let mut response = Vec::new();
    {
        let mut handle = Easy::new();   

        handle.url("http://localhost:8003").unwrap();
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

    assert_eq!(response.len(), 61);

    let response_text = String::from_utf8(response).unwrap();
    assert!(response_text.starts_with("HTTP/1.1 404 Not Found\r\n"));
    assert!(response_text.contains("\r\nServer: Osmium"));
    assert!(response_text.contains("\r\nContent-Length: 0"));
    assert!(response_text.ends_with("\r\n\r\n"));
}
