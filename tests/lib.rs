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
extern crate curl;
extern crate rustc_version;

// std
use std::{thread, time};
use std::env;

// curl
use curl::easy::Easy;

// osmium
use osmium::http::{request, response, server, handler, status, header};

fn get_server_port_range_start() -> i32 {
    match env::var("SERVER_PORT_RANGE_START") {
        Ok(val) => val.parse::<i32>().unwrap(),
        Err(e) => {
            match rustc_version::version_meta() {
                Ok(meta) => {
                    match meta.channel {
                        rustc_version::Channel::Stable => {
                            5000
                        },
                        rustc_version::Channel::Beta => {
                            6000
                        },
                        rustc_version::Channel::Nightly => {
                            7000
                        },
                        _ => {
                            info!("Unhandled rust channel, using default port");
                            8000
                        }
                    }
                },
                Err(_) => {
                    warn!("Server port range start is not set and cannot be determined from build channel. Using default");
                    8000
                }
            }
        }
    }
}

#[test]
fn empty_request() {
    let port = get_server_port_range_start() + 1;

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
            port: Some(port)
        };

        server::run(MyHandler, Some(settings));
    });

    let mut response = Vec::new();
    {
        let mut handle = Easy::new();   

        handle.url(format!("http://localhost:{}", port).as_str()).unwrap();
        handle.show_header(true).unwrap();
        let mut transfer = handle.transfer();
        transfer.write_function(|new_data| {
            trace!("Reading response line: [{:?}]", new_data);
            response.extend_from_slice(new_data);
            Ok(new_data.len())
        }).unwrap();
        debug!("Making a request to the server");

        for _ in 1..3 {
            match transfer.perform() {
                Ok(()) => {
                    break;
                },
                Err(e) => {
                    error!("Oops, the request failed. [{}]", e);
                    thread::sleep(time::Duration::from_secs(3));
                }
            }
        }
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

    let port = get_server_port_range_start() + 2;

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
            port: Some(port)
        };

        server::run(MyHandler, Some(settings));
    });

    let mut response = Vec::new();
    {
        let mut handle = Easy::new();   

        handle.url(format!("http://localhost:{}/index.html", port).as_ref()).unwrap();
        handle.show_header(true).unwrap();
        let mut transfer = handle.transfer();
        transfer.write_function(|new_data| {
            trace!("Reading response line: [{:?}]", new_data);
            response.extend_from_slice(new_data);
            Ok(new_data.len())
        }).unwrap();
        debug!("Making a request to the server");
        
        for _ in 1..3 {
            match transfer.perform() {
                Ok(()) => {
                    break;
                },
                Err(e) => {
                    error!("Oops, the request failed. [{}]", e);
                    thread::sleep(time::Duration::from_secs(3));
                }
            }
        }
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
    let port = get_server_port_range_start() + 3;

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
            port: Some(port)
        };

        server::run(MyHandler, Some(settings));
    });

    let mut response = Vec::new();
    {
        let mut handle = Easy::new();   

        handle.url(format!("http://localhost:{}", port).as_ref()).unwrap();
        handle.show_header(true).unwrap();
        let mut transfer = handle.transfer();
        transfer.write_function(|new_data| {
            trace!("Reading response line: [{:?}]", new_data);
            response.extend_from_slice(new_data);
            Ok(new_data.len())
        }).unwrap();
        debug!("Making a request to the server");
        
        for _ in 1..3 {
            match transfer.perform() {
                Ok(()) => {
                    break;
                },
                Err(e) => {
                    error!("Oops, the request failed. [{}]", e);
                    thread::sleep(time::Duration::from_secs(3));
                }
            }
        }
    }

    assert_eq!(response.len(), 61);

    let response_text = String::from_utf8(response).unwrap();
    assert!(response_text.starts_with("HTTP/1.1 404 Not Found\r\n"));
    assert!(response_text.contains("\r\nServer: Osmium"));
    assert!(response_text.contains("\r\nContent-Length: 0"));
    assert!(response_text.ends_with("\r\n\r\n"));
}
