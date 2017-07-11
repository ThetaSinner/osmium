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
extern crate futures;
extern crate tokio_service;
extern crate tokio_proto;
#[macro_use] extern crate log;
extern crate pretty_env_logger;

// std
use std::io;

// tokio
use futures::{future, Future, BoxFuture};
use tokio_service::Service;
use tokio_proto::TcpServer;

// osmium
use osmium::http::{codec, request, response};

#[test]
fn serve_http_initial_test() {
    pretty_env_logger::init().unwrap();

    struct InitialTestService;

    impl Service for InitialTestService {
        type Request = request::Request;
        type Response = response::Response;
        type Error = io::Error;
        type Future = BoxFuture<response::Response, io::Error>;

        fn call(&self, req: Self::Request) -> Self::Future {
            let response = response::Response {
                version: req.version
            };

            future::ok(response).boxed()
        }
    }

    // Specify the localhost address
    let addr = "0.0.0.0:2234".parse().unwrap();

    // The builder requires a protocol and an address
    let server = TcpServer::new(codec::Http, addr);

    // We provide a way to *instantiate* the service for each new
    // connection; here, we just immediately return a new instance.
    server.serve(|| Ok(InitialTestService));
}
