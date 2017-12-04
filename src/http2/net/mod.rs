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

pub mod h2handshake;
pub mod https;
pub mod acceptor_factory;
pub mod shutdown_signal;

// std
use std::sync::mpsc;
use std::sync::Arc;
use std::rc::Rc;
use std::marker;
use std::mem;
use std::net;

// tokio
use futures::{Stream, Sink, Future, stream};
use futures::future::{self, loop_fn};
use futures::sync::mpsc as futures_mpsc;
use futures::sync::oneshot as futures_oneshot;
use tokio_core;
use tokio_io::io as tokio_io;
use tokio_io::AsyncRead;

// threadpool
use threadpool::ThreadPool;
use std::convert;

// osmium
use http2::frame as framing;
use http2::core::connection;
use http2::hpack;
use shared::server_trait;
use http2::stream as streaming;
use http2::settings;
use shared::server_settings;

#[derive(Debug)]
pub enum ServerError {
    InvalidSettingsConfiguration
}

// TODO this doesn't really belong in the net package.
pub struct Server<T, R, S>
    where T: server_trait::OsmiumServer<Request=R, Response=S>, 
          R: convert::From<streaming::StreamRequest>,
          S: convert::Into<streaming::StreamResponse>
{
    hpack: hpack::HPack,
    app: T,
    security_settings: Option<server_settings::SecuritySettings>,
    bind_address: net::SocketAddr,
    local_settings: settings::Settings,
    local_settings_frame: framing::settings::SettingsFrameCompressModel
}

impl<T, R, S> Server<T, R, S> 
    where T: 'static + server_trait::OsmiumServer<Request=R, Response=S> + marker::Sync + marker::Send,
          R: 'static + convert::From<streaming::StreamRequest>,
          S: 'static + convert::Into<streaming::StreamResponse>
{
    pub fn new(app: T, server_settings: server_settings::ServerSettings) -> Result<Self, ServerError> {
        // Read the settings configuration.
        let mut local_settings = settings::Settings::spec_default();
        if local_settings.apply_changes(server_settings.get_http2_settings()).is_err() {
            return Err(ServerError::InvalidSettingsConfiguration);
        };

        let mut local_settings_frame = framing::settings::SettingsFrameCompressModel::new();
        for setting in server_settings.get_http2_settings() {
            local_settings_frame.add_parameter(setting.get_name(), setting.get_value());
        }

        // Build the server bind address.
        let addr = format!("{}:{}", server_settings.get_host(), server_settings.get_port()).parse().unwrap();

        Ok(Server {
            hpack: hpack::HPack::new(),
            app: app,
            security_settings: server_settings.get_security(),
            bind_address: addr,
            local_settings: local_settings,
            local_settings_frame: local_settings_frame
        })
    }

    // The start method consumes self so that it can ensure it can be used on the connection threads.
    // The connection threads take a closure which must have static lifetime. If server startup 
    // succeeds, then the a shared pointer to self is returned.
    pub fn start_server(mut self) -> Arc<Box<Self>>
    {
        // tokio event loop
        let mut event_loop = tokio_core::reactor::Core::new().unwrap();
        let handle = event_loop.handle();

        // create a listener for incoming tcp connections
        let listener = tokio_core::net::TcpListener::bind(&self.bind_address, &handle).unwrap();

        let thread_pool = Rc::new(ThreadPool::new(10));

        // Create the acceptor factory.
        let mut security = None;
        mem::swap(&mut security, &mut self.security_settings);
        // TODO might not exist.
        let acceptor_factory = acceptor_factory::AcceptorFactory::new(&security.unwrap());

        // TODO this should vary depending on the startup type chosen (http/https)
        let handshake: Box<self::h2handshake::H2Handshake> = Box::new(https::HttpsH2Handshake::new(acceptor_factory));

        let server_instance = Arc::new(Box::new(self));

        // get a stream (infinite iterator) of incoming connections
        let server = listener.incoming().zip(stream::repeat(server_instance.clone())).for_each(|((socket, _remote_addr), server_instance)| {
            debug!("Starting connection on {}", _remote_addr);

            let inner_handle = handle.clone();
            
            let handshake_future = handshake.attempt_handshake(socket, Box::new(server_instance.local_settings_frame.clone()))
            .map_err(|e| {
                error!("I/O error while attempting connection handshake {}", e);
            })
            .join(Ok(thread_pool.clone()))
            .map(move |(handshake_result, thread_pool)| {
                // Convert the future result to a standard result.
                let handshake_result = handshake_result
                .map_err(|handshake_error| {
                    handshake_error
                })
                .map(|handshake_completion| {
                    handshake_completion
                })
                .wait(); // safe to wait here as long as the handshake result is a future result.
                
                match handshake_result {
                    Ok(mut handshake_completion) => {
                        // TODO (naming) really? temp_frame... fix me
                        let mut temp_frame = framing::settings::SettingsFrame::new_noop();
                        mem::swap(&mut handshake_completion.settings_frame, &mut temp_frame);
                        
                        let (reader, writer) = handshake_completion.stream.split();

                        let (shutdown_read_tx, shutdown_read_rx) = futures_oneshot::channel::<u8>();
                        let (mut ftx, frx) = futures_mpsc::channel(5);
                        let (tx, rx) = mpsc::channel::<(framing::FrameHeader, Vec<u8>)>();
                        thread_pool.execute(move || {
                            let mut connection = connection::Connection::new(
                                server_instance.hpack.new_send_context(),
                                server_instance.hpack.new_recv_context(),
                                server_instance.local_settings.clone(),
                                temp_frame,
                                shutdown_signal::ShutdownSignaller::new(shutdown_read_tx)
                            );

                            // Note that if the initial settings contain an error the connection will immediately initiate shutdown.
                            // The documentation for iter() on a receiver states that it will never panic, just yield None when the 
                            // sender has hung up.
                            let mut msg_iter = rx.iter();
                            while let Some(msg) = msg_iter.next() {
                                connection.recv(
                                    framing::Frame {
                                        header: msg.0,
                                        payload: msg.1
                                    },
                                    &server_instance.app
                                );
                                
                                while let Some(response_frame) = connection.pull_frame() {
                                    // TODO it's possible to crash the server here... no idea how but this can't just be unwrapped.
                                    ftx = ftx.send(response_frame).wait().unwrap();
                                }

                                // TODO this is quite a big commitement, the connection will not process any new frames until this is done.
                                // Of course, frames will still be read off the network and queued for when this finishes.
                                while connection.execute_promised(&server_instance.app) {
                                    while let Some(response_frame) = connection.pull_frame() {
                                        ftx = ftx.send(response_frame).wait().unwrap();
                                    }
                                }
                            }

                            println!("about to drop connection");

                            // TODO (goaway) The loop has exited (again, hopefully) so check for send frames and figure out how to
                            // keep the send loop alive long enough to make sure the goaway frame has sent.
                            // TODO (goaway) Then make sure the send loop shuts down. When ftx is dropped the loop should end but it needs
                            // to be checked.
                            // Now just letting this closure exit will free this thread back into the pool to be used again.
                        });

                        let shutdown_read_future = shutdown_read_rx.map(|_| [
                            // value received on shutdown oneshot, ignore value.
                            ()
                        ]);

                        let reader_loop = loop_fn((reader, tx, shutdown_read_future), move |(reader, to_conn_thread, shutdown_read_future)| {
                            // this read exact will run on the event loop until enough bytes for an
                            // http2 header frame have been read
                            let read_frame_future = tokio_io::read_exact(reader, [0; framing::FRAME_HEADER_SIZE])
                                .and_then(|(reader, frame_header_buf)| {
                                    // Note that this can panic if the frame header is too small. But because the first read specifies
                                    // how many bytes to read before continuing to here, that should never be a problem.
                                    let frame_header = framing::decompress_frame_header(frame_header_buf.to_vec());
                                    
                                    let mut buf = Vec::with_capacity(frame_header.length as usize);
                                    buf.resize(frame_header.length as usize, 0);

                                    tokio_io::read_exact(reader, buf).join(future::ok(frame_header))
                                });

                            let loop_read_frame_future = read_frame_future
                                .join(future::ok(to_conn_thread));

                            loop_read_frame_future.select2(shutdown_read_future).then(|result| {
                                match result {
                                    Ok(future::Either::A(((((reader, payload_buf), frame_header), to_conn_thread), shutdown_read_future))) => {
                                        trace!("got frame [{:?}]: [{:?}]", frame_header, payload_buf);

                                        to_conn_thread.send((frame_header, payload_buf)).unwrap();

                                        Ok(future::Loop::Continue((reader, to_conn_thread, shutdown_read_future)))
                                    },
                                    Ok(future::Either::B((_, _read_future))) => {
                                        debug!("The internal connection has sent the shutdown signal to the network read loop, nothing more will be read.");
                                        Ok(future::Loop::Break(()))
                                    },
                                    Err(future::Either::A((e, _))) => {
                                        error!("Connection terminated by the remote [{}]", e);
                                        Ok(future::Loop::Break(()))
                                    },
                                    Err(future::Either::B((e, _))) => {
                                        error!("Network read loop lost connection with the internal connection, will stop reading from the network [{:?}]", e);
                                        Ok(future::Loop::Break(()))
                                    }
                                }
                            })
                        });

                        inner_handle.spawn(reader_loop);

                        // From the documentation, when all sender handles have been dropped the stream is considered completed and 'none' is
                        // returned. That is what is needed to end the 'fold'.
                        // Therefore, no shutdown mechanism is required for this. As soon as the connection thread exits this future will complete
                        // and be removed from the event loop.
                        let send_loop = frx.fold(writer, |writer, msg| {
                            trace!("will push to network [{:?}]", msg);
                            tokio_io::write_all(writer, msg)
                                .map(|(w, _)| {
                                    // Yield the writer to the next iteration of this process.
                                    w
                                })
                                .map_err(|e| {
                                    // TODO If there's a network error then the connection should probably be shut down?
                                    error!("error writing to the network [{:?}]", e);
                                    ()
                                })
                        })
                        .map(|_| ()); // Drop the writer, can't move a future which returns a value onto the event loop.

                        inner_handle.spawn(send_loop);
                    },
                    Err(e) => {
                        match e {
                            h2handshake::HandshakeError::DidNotUpgrade(_connection, received_bytes) => {
                                if received_bytes.len() > 80 {
                                    info!("Rejected connection because of failed handshake [{:?}]", &received_bytes[0..80]);
                                }
                                else {
                                    info!("Rejected connection because of failed handshake [{:?}]", received_bytes);
                                }
                            }
                        }
                    }
                }
            });

            handle.spawn(handshake_future);
            
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

    use http2::header;
    use http2::stream as streaming;
    use shared::{server_trait, server_settings};
    use shared::connection_handle::ConnectionHandle;

    struct MyServer;

    #[derive(Debug)]
    struct HttpRequest {
        pub headers: header::Headers,
        pub body: Option<String>
    }

    #[derive(Debug)]
    struct HttpResponse {
        pub headers: header::Headers,
        pub body: Option<Vec<u8>>
    }

    impl From<streaming::StreamRequest> for HttpRequest {
        fn from(stream_request: streaming::StreamRequest) -> HttpRequest {
            let body = if stream_request.payload.is_some() {
                Some(
                    String::from_utf8(stream_request.payload.unwrap()).unwrap()
                )
            }
            else {
                None
            };

            HttpRequest {
                headers: stream_request.headers,
                body: body
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

    impl server_trait::OsmiumServer for MyServer {
        type Request = HttpRequest;
        type Response = HttpResponse;

        fn process(&self, request: Self::Request, handle: Box<&mut ConnectionHandle>) -> Self::Response {
            println!("Got request {:?}", request);

            let mut headers = header::Headers::new();
            headers.push(header::HeaderName::PseudoStatus, header::HeaderValue::Num(200));
            headers.push(header::HeaderName::ContentLength, header::HeaderValue::Num(111));
            headers.push(header::HeaderName::ContentType, header::HeaderValue::Str(String::from("text/html")));

            HttpResponse {
                headers: headers,
                body: Some(String::from("<!DOCTYPE html><html><head><title>test</title></head><body><h1>Osmium served me like a beast</h1></body></html>").into_bytes())
            }
        }
    }

    // MANUAL TESTING #[test]
    fn test_start_server() {
        println!("start server");
        let mut settings = server_settings::ServerSettings::default();
        settings.set_security(server_settings::SecuritySettings::default());

        Server::new(MyServer {}, settings).unwrap().start_server();
    }
}
