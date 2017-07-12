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
use std::marker;
use std::sync::Arc;

// tokio
use tokio_proto::TcpServer;
use futures_cpupool::CpuPool;

// osmium
use http::{handler, http_protocol, http_service};

pub struct Settings {
    pub host: Option<String>,
    pub port: Option<i32>,
}

pub fn run<T>(handler: T, settings: Option<Settings>) where T: handler::Handler + marker::Send + marker::Sync + 'static {
    let cpu_pool = CpuPool::new(10);
    let handler_ref = Arc::new(handler);

    let addr = (if let Some(settings) = settings {
        format!("{}:{}", 
        match settings.host {Some(host) => host, None => "0.0.0.0".to_owned()},
        match settings.port {Some(port) => port, None => 8000}
        )
    }
    else {
        "0.0.0.0:80".to_owned()
    }).parse().unwrap();

    // The builder requires a protocol and an address
    let server = TcpServer::new(http_protocol::HttpProtocol, addr);

    server.serve(move || {
        Ok(http_service::HttpService {
            cpupool: cpu_pool.clone(),
            handler: handler_ref.clone()
        })
    });
}
