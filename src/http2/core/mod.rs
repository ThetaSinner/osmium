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

// TODO rename this module to connection or similar.

mod connection_frame_state;

// std
use std::collections::{VecDeque, hash_map, HashMap};
use std::convert;
use std::cell::RefCell;
use std::rc::Rc;

// osmium
use http2::frame as framing;
use http2::error;
use http2::stream as streaming;
use http2::hpack::context as hpack_context;
use shared::server_trait;
use http2::settings;

pub struct ConnectionData {
    pub incoming_settings: settings::Settings,
    next_server_created_stream_id: u32 // TODO typedef
}

impl ConnectionData {
    pub fn new() -> Self {
        ConnectionData {
            incoming_settings: settings::Settings::spec_default(),
            next_server_created_stream_id: 2
        }
    }

    // TODO This could be named better
    pub fn get_next_server_created_stream_id(&mut self) -> u32 {
        let id = self.next_server_created_stream_id;
        self.next_server_created_stream_id += 2;
        id
    }
}

pub struct Connection<'a> {
    send_frames: VecDeque<Vec<u8>>,
    frame_state_validator: connection_frame_state::ConnectionFrameStateValidator,

    hpack_send_context: hpack_context::SendContext<'a>,
    hpack_recv_context: hpack_context::RecvContext<'a>,

    streams: HashMap<framing::StreamId, streaming::Stream>,

    promised_streams_queue: VecDeque<framing::StreamId>,

    connection_data: Rc<RefCell<ConnectionData>>,

    send_window: u32
}

impl<'a> Connection<'a> {
    pub fn new(hpack_send_context: hpack_context::SendContext<'a>, hpack_recv_context: hpack_context::RecvContext<'a>) -> Connection<'a> {
        Connection {
            send_frames: VecDeque::new(),
            frame_state_validator: connection_frame_state::ConnectionFrameStateValidator::new(),
            hpack_send_context: hpack_send_context,
            hpack_recv_context: hpack_recv_context,
            streams: HashMap::new(),
            promised_streams_queue: VecDeque::new(),
            connection_data: Rc::new(RefCell::new(ConnectionData::new())),
            send_window: 0
        }
    }

    pub fn push_frame<T, R, S>(&mut self, frame: framing::Frame, app: &T)
        where T: server_trait::OsmiumServer<Request=R, Response=S>, 
              R: convert::From<streaming::StreamRequest>,
              S: convert::Into<streaming::StreamResponse>
    {
        log_conn_frame!("Receive frame", frame);

        // TODO handle frame type not recognised.
        let frame_type = match frame.header.frame_type {
            Some(ref frame_type) => frame_type.clone(),
            None => {
                // TODO this can be handled gracefully, no need to crash.
                panic!("cannot handle frame type not recognised");
            }
        };

        // Check that the incoming frame is what was expected on this connection.
        if !self.frame_state_validator.is_okay(frame_type.clone(), frame.header.flags, frame.header.stream_id) {
            // (6.2) A receiver MUST treat the receipt of any other type of frame 
            // or a frame on a different stream as a connection error (Section 5.4.1) 
            // of type PROTOCOL_ERROR.
            self.push_send_go_away_frame(error::HttpError::ConnectionError(
                error::ErrorCode::ProtocolError,
                error::ErrorName::HeaderBlockInterupted
            ));
            return;
        };

        match frame_type {
            framing::FrameType::Ping => {
                // (6.7) A PING frame with a stream identifier other than 0x0 is a connection error of type PROTOCOL_ERROR
                if frame.header.stream_id != 0x0 {
                    self.push_send_go_away_frame(error::HttpError::ConnectionError(
                        error::ErrorCode::ProtocolError,
                        error::ErrorName::StreamIdentifierOnConnectionFrame
                    ));
                    // (6.7) If a PING frame with a stream identifier other than 0x0 is received, then the recipient must
                    // respond with a connection error.
                    // That is, do not continue processing the ping.
                    return;
                }

                // decode the incoming ping frame
                let ping_frame_result = framing::ping::PingFrame::new(&frame.header, &mut frame.payload.into_iter());

                match ping_frame_result {
                    Ok(ping_frame) => {
                        if framing::ping::is_acknowledge(frame.header.flags) {
                            // TODO the server has no way of managing the connection thread. That is, the thread is only
                            // active when frames are received which means the connection is active and there's no point
                            // sending a ping.
                            panic!("can't handle ping response");
                        }
                        else {
                            // TODO add a second constructor method which builds a response.
                            let mut ping_response = framing::ping::PingFrameCompressModel::new();
                            ping_response.set_acknowledge();
                            ping_response.set_ping_payload(ping_frame.get_payload());

                            // (6.7) A PING frame with a stream identifier other than 0x0 is a connection error of type PROTOCOL_ERROR
                            self.push_send_frame(Box::new(ping_response), 0x0);
                        }
                    },
                    Err(e) => {
                        // (6.7) the only error which decoding can produce is a FRAME_SIZE_ERROR, which is a connection error
                        // so it is correct to build a GO_AWAY frame from it.
                        self.push_send_go_away_frame(e);
                    }
                }
            },
            framing::FrameType::Headers => {
                // (6.2) A HEADERS frame which is not associated with a stream is a connection error of type PROTOCOL_ERROR
                if frame.header.stream_id == 0x0 {
                    self.push_send_go_away_frame(error::HttpError::ConnectionError(
                        error::ErrorCode::ProtocolError,
                        error::ErrorName::MissingStreamIdentifierOnStreamFrame
                    ));
                    return;
                }

                self.move_to_stream(frame_type, frame, app);
            },
            framing::FrameType::Data => {
                if frame.header.stream_id == 0x0 {
                    self.push_send_go_away_frame(error::HttpError::ConnectionError(
                        error::ErrorCode::ProtocolError,
                        error::ErrorName::MissingStreamIdentifierOnStreamFrame
                    ));
                    return;
                }

                self.move_to_stream(frame_type, frame, app);
            },
            framing::FrameType::WindowUpdate => {
                // TODO would be nice if this was a named operation.
                if frame.header.stream_id == 0x0 {
                    let window_update_frame = framing::window_update::WindowUpdateFrame::new_conn(&frame.header, &mut frame.payload.into_iter());

                    // TODO handle frame decode error.

                    if window_update_frame.get_window_size_increment() == 0 {
                        self.push_send_go_away_frame(error::HttpError::ConnectionError(
                            error::ErrorCode::ProtocolError,
                            error::ErrorName::ZeroWindowSizeIncrement
                        ));
                    }
                    else {
                        self.send_window += window_update_frame.get_window_size_increment();
                    }
                }
                else {
                    self.move_to_stream(frame_type, frame, app);
                }
            },
            framing::FrameType::Priority => {
                if frame.header.stream_id == 0x0 {
                    self.push_send_go_away_frame(error::HttpError::ConnectionError(
                        error::ErrorCode::ProtocolError,
                        // TODO this means a client using this error as a debug message won't know which frame caused a problem
                        error::ErrorName::MissingStreamIdentifierOnStreamFrame
                    ));
                    return;
                }

                // Do not process this frame yet. Priority isn't a feature that's required at all, especially in 
                // an initial version of this server.
            },
            framing::FrameType::Settings => {
                if frame.header.stream_id != 0x0 {
                    self.push_send_go_away_frame(error::HttpError::ConnectionError(
                        error::ErrorCode::ProtocolError,
                        // TODO this means a client using this error as a debug message won't know which frame caused a problem
                        error::ErrorName::StreamIdentifierOnConnectionFrame
                    ));
                    return;
                }

                let settings_frame = framing::settings::SettingsFrame::new(&frame.header, &mut frame.payload.into_iter());

                self.apply_settings(settings_frame);
            },
            framing::FrameType::GoAway => {
                let go_away_frame = framing::go_away::GoAwayFrame::new(&frame.header, &mut frame.payload.into_iter());

                println!("go away frame received from client {:?}", go_away_frame);

                panic!("will crash; did not expect go away from client");
            },
            _ => {
                panic!("can't handle that frame type yet {:?}", frame_type);
            }
        }
    }

    pub fn execute_promised<T, R, S>(&mut self, app: &T) -> bool
        where T: server_trait::OsmiumServer<Request=R, Response=S>, 
              R: convert::From<streaming::StreamRequest>,
              S: convert::Into<streaming::StreamResponse>
    {
        if let Some(promised_stream_id) = self.promised_streams_queue.pop_back() {
            let mut temp_streams = Vec::new();
            {
                let stream = self.streams.entry(promised_stream_id);

                match stream {
                    hash_map::Entry::Occupied(mut stream) => {
                        let stream = stream.get_mut();

                        stream.recv_promised(&mut self.hpack_send_context, app);

                        while let Some((promised_stream_id, stream_request)) = stream.fetch_push_promise() {
                            let promise_stream = streaming::Stream::new_promise(promised_stream_id, self.connection_data.clone(), stream_request);

                            temp_streams.push((promised_stream_id, promise_stream));
                            self.promised_streams_queue.push_front(promised_stream_id);
                        }

                        // Fetch any send frames which have been generated on the stream.
                        self.send_frames.extend(stream.fetch_send_frames());
                    },
                    hash_map::Entry::Vacant(_) => {
                        panic!("expected reserved stream, but nothing was found");
                    }
                }
            }
            
            while let Some((promised_stream_id, promised_stream)) = temp_streams.pop() {
                self.streams.insert(promised_stream_id, promised_stream);
            }
            
            true
        }
        else {
            false
        }
    }

    fn move_to_stream<T, R, S>(&mut self, frame_type: framing::FrameType, frame: framing::Frame, app: &T)
        where T: server_trait::OsmiumServer<Request=R, Response=S>,
              R: convert::From<streaming::StreamRequest>,
              S: convert::Into<streaming::StreamResponse>
    {
        // TODO this methods is a mess because it needs to borrow self.streams twice. Make it better.

        let mut temp_streams = Vec::new();
        {
            let stream = self.streams
                .entry(frame.header.stream_id)
                .or_insert(streaming::Stream::new(frame.header.stream_id, self.connection_data.clone()));

            let stream_response = stream.recv(
                framing::StreamFrame {
                    // TODO constructor for converting the header.
                    header: framing::StreamFrameHeader {
                        length: frame.header.length,
                        frame_type: frame_type,
                        flags: frame.header.flags
                    },
                    payload: frame.payload
                },
                &mut self.hpack_send_context,
                &mut self.hpack_recv_context,
                app
            );

            // TODO handle the error. Because it might kill the stream or the connection, it cannot be ignored.
            if let Some(err) = stream_response {
                error!("Error on stream {}. The error was {:?}", frame.header.stream_id, err);
            }

            // TODO does the stream build its error or does the error frame get built and sent here.

            // For each push promise, creates a new stream which is in the reserved state and queues that new stream
            // for processing later.
            while let Some((promised_stream_id, stream_request)) = stream.fetch_push_promise() {
                let promise_stream = streaming::Stream::new_promise(promised_stream_id, self.connection_data.clone(), stream_request);

                temp_streams.push((promised_stream_id, promise_stream));
                self.promised_streams_queue.push_front(promised_stream_id);
            }

            // Fetch any send frames which have been generated on the stream.
            self.send_frames.extend(stream.fetch_send_frames());
        }

        while let Some((promised_stream_id, promised_stream)) = temp_streams.pop() {
            self.streams.insert(promised_stream_id, promised_stream);
        }
    }

    // Queues a frame to be sent.
    fn push_send_frame(&mut self, frame: Box<framing::CompressibleHttpFrame>, stream_id: framing::StreamId) {
        log_conn_send_frame!("Pushing frame for send", frame);

        self.send_frames.push_back(
            framing::compress_frame(frame, stream_id)
        );
    }

    fn push_send_go_away_frame(&mut self, http_error: error::HttpError) {
        // TODO send last stream processed. Steams are not implemented yet so this will have to wait. For now send
        // 0x0, which means no streams processed.
        let go_away = framing::go_away::GoAwayFrameCompressModel::new(0x0, http_error);

        // (6.8) A GOAWAY frame with a stream identifier other than 0x0 is a connection error of type PROTOCOL_ERROR.
        self.push_send_frame(Box::new(go_away), 0x0);
    }

    // TODO do a fetch all like in stream?
    pub fn pull_frame(&mut self) -> Option<Vec<u8>> {
        self.send_frames.pop_front()
    }

    fn apply_settings(&mut self, settings_frame: framing::settings::SettingsFrame) {
        for setting in settings_frame.get_parameters() {
            match setting.get_name() {
                &settings::SettingName::SettingsHeaderTableSize => {
                    // This sets the maximum size that the local encoder can use. 
                    //
                    // The remote encoder can reduce the space it's using and communicate that 
                    // reduction within hpack. So if the remote decoder needs to use less space, 
                    // then it must update this setting.
                    // 
                    // Therefore, when this setting arrives the local encoder must be notified.
                    // The encoder will then reduce the size it's using as the first instruction
                    // in the next header block.

                    // TODO Currently, this saves the current value, but probably isn't necesary.
                    self.connection_data.borrow_mut().incoming_settings.header_table_size = setting.get_value();

                    // Inform the send context that the max size setting has changed.
                    self.hpack_send_context.inform_max_size_setting_changed(self.connection_data.borrow().incoming_settings.header_table_size);
                },
                &settings::SettingName::SettingsEnablePush => {
                    match setting.get_value() {
                        0 => {
                            // TODO when push promise is disabled remotely then any streams which
                            // are reserved remote need to be reset.
                            self.connection_data.borrow_mut().incoming_settings.enable_push = false;
                        },
                        1 => {
                            // There is nothing to be done when this setting is switched on. The next
                            // time the application wants to push promise it will be enabled.
                            self.connection_data.borrow_mut().incoming_settings.enable_push = true;
                        },
                        _ => {
                            // (6.5.2) Any value other than 0 or 1 MUST be treated as a connection 
                            // error (Section 5.4.1) of type PROTOCOL_ERROR.
                            self.push_send_go_away_frame(
                                error::HttpError::ConnectionError(
                                    error::ErrorCode::ProtocolError,
                                    error::ErrorName::EnablePushSettingInvalidValue
                                )
                            );
                        }
                    }
                },
                &settings::SettingName::SettingsMaxConcurrentStreams => {
                    // TODO need to refuse to open new streams if it would exceed the remote limit (for a server this is just limiting the number of push promises)
                    // TODO need to send reset stream with stream refused if the client exceeds the limit we've set. If the client continues to try to open streams
                    // very quickly while open streams are sill being processed then we can send reset with enhance your calm :)
                    self.connection_data.borrow_mut().incoming_settings.max_concurrent_streams = Some(setting.get_value());
                },
                &settings::SettingName::SettingsInitialWindowSize => {
                    let val = setting.get_value();

                    if val <= settings::MAXIMUM_FLOW_CONTROL_WINDOW_SIZE {
                        self.connection_data.borrow_mut().incoming_settings.initial_window_size = val;

                        // This is the window size that new streams will use.

                        // TODO this also affects some existing streams, see (6.9.2) and (6.9.3)
                    }
                    else {
                        // (6.5.2) Values above the maximum flow-control window size of 231-1 MUST be treated as a 
                        // connection error (Section 5.4.1) of type FLOW_CONTROL_ERROR.
                        self.push_send_go_away_frame(
                            error::HttpError::ConnectionError(
                                error::ErrorCode::ProtocolError,
                                error::ErrorName::InvalidInitialWindowSize
                            )
                        );
                    }
                },
                &settings::SettingName::SettingsMaxFrameSize => {
                    let val = setting.get_value();

                    if settings::INITIAL_MAX_FRAME_SIZE <= val && val <= settings::MAXIMUM_MAX_FRAME_SIZE {
                        // TODO if a frame payload which is too big to send with the current limit then it is necessary to block
                        // locally. Hopefully the remote will receive the response headers and realise the need to increase this 
                        // setting value, at which point we need to trigger and event to check all responses blocked for this reason.
                        // TODO handle the local side of the above, if we can't receive a payload make a decision about whether
                        // to increase this setting to allow the remote to send its payload.
                        self.connection_data.borrow_mut().incoming_settings.max_frame_size = val;
                    }
                    else {
                        // (6.5.2) The initial value is 214 (16,384) octets. The value advertised by an endpoint MUST be between this initial 
                        // value and the maximum allowed frame size (224-1 or 16,777,215 octets), inclusive. Values outside this range MUST 
                        // be treated as a connection error (Section 5.4.1) of type PROTOCOL_ERROR.
                        self.push_send_go_away_frame(
                            error::HttpError::ConnectionError(
                                error::ErrorCode::ProtocolError,
                                error::ErrorName::InvalidMaxFrameSize
                            )
                        );
                    }
                },
                &settings::SettingName::SettingsMaxHeaderListSize => {
                    // TODO no idea how to handle exceeding this limit on send.
                    self.connection_data.borrow_mut().incoming_settings.max_header_list_size = Some(setting.get_value());
                }
            }
        }

        // TODO acknowledge the settings by sending a setting acknowledge frame.
    }
}

pub enum PushError {
    TooManyActiveStreams
}

use http2::stream::StreamRequest;

pub trait ConnectionHandle {
    fn is_push_enabled(&self) -> bool;

    fn push_promise(&mut self, request: StreamRequest) -> Option<PushError>;
}
