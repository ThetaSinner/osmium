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

// std
use std::sync::mpsc;
use std::sync::Arc;

// tokio
use futures::{Stream, Sink, Future, stream};
use futures::future::{self, loop_fn, Loop};
use futures::sync::mpsc as futures_mpsc;
use tokio_core;
use tokio_io::io as tokio_io;
use tokio_io::AsyncRead;

// threadpool
use threadpool::ThreadPool;

// osmium
use http2::frame as framing;
use http2::core;
use http2::hpack;

struct Server {
    hpack: hpack::HPack,
}

impl Server {
    pub fn new() -> Self {
        Server {
            hpack: hpack::HPack::new()
        }
    }

    // The start method consumes self so that it can ensure it can be used on the connection threads.
    // The connection threads take a closure which must have static lifetime. If server startup 
    // succeeds, then the a shared pointer to self is returned.
    pub fn start_server(self) -> Arc<Box<Self>> {
        // tokio event loop
        let mut event_loop = tokio_core::reactor::Core::new().unwrap();
        let handle = event_loop.handle();

        // create a listener for incoming tcp connections
        let addr = "127.0.0.1:8080".parse().unwrap();
        let listener = tokio_core::net::TcpListener::bind(&addr, &handle).unwrap();

        let thread_pool = ThreadPool::new(10);

        let server_instance = Arc::new(Box::new(self));

        // get a stream (infinite iterator) of incoming connections
        let server = listener.incoming().zip(stream::repeat(server_instance.clone())).for_each(|((socket, _remote_addr), server_instance)| {
            debug!("Starting connection on {}", _remote_addr);

            let (reader, writer) = socket.split();

            let (mut ftx, frx) = futures_mpsc::channel(5);
            let (tx, rx) = mpsc::channel::<(framing::FrameHeader, Vec<u8>)>();
            thread_pool.execute(move || {
                let mut connection = core::Connection::new(server_instance.hpack.new_context());

                let mut msg_iter = rx.iter();
                while let Some(msg) = msg_iter.next() {
                    connection.push_frame(
                        framing::Frame {
                            header: msg.0,
                            payload: msg.1
                        }
                    );
                    
                    while let Some(response_frame) = connection.pull_frame() {
                        ftx = ftx.send(response_frame).wait().unwrap();
                    }
                }
            });

            let reader_loop = loop_fn((reader, tx), move |(reader, to_conn_thread)| {
                // this read exact will run on the event loop until enough bytes for an
                // http2 header frame have been read

                let read_frame_future = tokio_io::read_exact(reader, [0; framing::FRAME_HEADER_SIZE])
                    .map_err(|err| {
                        // TODO this prints then swallows any errors. should handle any io errors
                        // handle error: connection closed results in unexpected eof error here
                        error!("Error reading the frame header [{:?}]", err);
                        ()
                    })
                    .and_then(|(reader, frame_header_buf)| {                   
                        let frame_header = framing::decompress_frame_header(frame_header_buf.to_vec());
                        
                        let mut buf = Vec::with_capacity(frame_header.length as usize);
                        buf.resize(frame_header.length as usize, 0);
                        tokio_io::read_exact(reader, buf)
                            .map_err(|err| {
                                // TODO handle the error
                                error!("Error reading the frame payload [{:?}]", err);
                                ()
                            })
                            .join(future::ok(frame_header))
                    });

                read_frame_future
                    .join(future::ok(to_conn_thread))
                    .and_then(|(((reader, payload_buf), frame_header), to_conn_thread)| {
                        trace!("got frame [{:?}]: [{:?}]", frame_header, payload_buf);

                        to_conn_thread.send((frame_header, payload_buf)).unwrap();

                        Ok(Loop::Continue((reader, to_conn_thread.clone())))
                    })
            });

            handle.spawn(reader_loop);

            let send_loop = frx.fold(writer, |writer, msg| {
                println!("will push to network [{:?}]", msg);
                tokio_io::write_all(writer, msg)
                    .map(|(w, _)| {
                        w
                    })
                    .map_err(|_e| {
                        error!("error writing to the network [{:?}]", _e);
                        ()
                    })
            }).map(|_| ());

            handle.spawn(send_loop);

            Ok(())
        });

        // move the incoming connection stream onto the event loop
        event_loop.run(server).unwrap();

        server_instance
    }
}

#[cfg(test)]
mod tests {
    use super::Server;

    // MANUAL TESTING #[test]
    fn test_start_server() {
        println!("start server");
        Server::new().start_server();
    }
}
