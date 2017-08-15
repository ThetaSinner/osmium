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
// along with Osmium. If not, see <http://www.gnu.org/licenses/>.

// tokio
use futures::{Stream, Future};
use futures::future::{loop_fn, Loop};
use tokio_core;
use tokio_io::io::read_exact;
use tokio_io::AsyncRead;

// threadpool
use threadpool;

pub fn start_server() {
    // tokio event loop
    let mut event_loop = tokio_core::reactor::Core::new().unwrap();
    let handle = event_loop.handle();

    // create a listener for incoming tcp connections
    let addr = "127.0.0.1:8080".parse().unwrap();
    let listener = tokio_core::net::TcpListener::bind(&addr, &handle).unwrap();

    // create a thread pool for incoming connections to run on
    let thread_pool = threadpool::ThreadPool::new(10);

    // get a stream (infinite iterator) of incoming connections
    let server = listener.incoming().for_each(|(socket, _remote_addr)| {
        debug!("Starting connection on {}", _remote_addr);

        let (reader, writer) = socket.split();

        let reader_loop = loop_fn(reader, |reader| {
            // this read exact will run on the event loop until enough bytes for an
            // http2 header frame have been read

            use http2::frame;

            read_exact(reader, [0; frame::FRAME_HEADER_SIZE])
                .map_err(|err| {
                    // TODO this prints then swallows any errors. should handle any io errors
                    println!("Err {:?}", err);
                    ()
                })
                .and_then(move |(reader, frame_header_buf)| {
                    println!("input data {:?}", frame_header_buf);

                    let frame_header = frame::decompress_frame_header(frame_header_buf.to_vec());

                    println!("{:?}", frame_header);

                    Ok(Loop::Continue(reader))
                })
        });

        handle.spawn(reader_loop);

        Ok(())
    });

    // move the incoming connection stream onto the event loop
    event_loop.run(server).unwrap();
}

#[cfg(test)]
mod tests {
    use super::start_server;

    #[test]
    fn test_start_server() {
        println!("call start server");
        start_server();
    }
}
