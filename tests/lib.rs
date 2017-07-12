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

// osmium
use osmium::http::{request, response, server, handler};

#[test]
fn serve_http_initial_test() {
    pretty_env_logger::init().unwrap();

    debug!("Starting a server");

    struct MyHandler;

    impl handler::Handler for MyHandler {
        fn process(&self, req: request::Request) -> response::Response {
            response::Response {
                version: req.version
            }
        }
    }

    server::run(MyHandler, None);
}
