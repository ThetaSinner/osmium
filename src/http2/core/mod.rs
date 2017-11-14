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
mod flow_control;
pub mod connection_shared_state;
mod stream_blocker;

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

// TODO (goaway) This connection struct doesn't have a unified way of processing frames on send. Need to look at
// how goaway frames are sent, from here and from streams. When a goaway is sent the net code needs to start shutdown
// and stop processing.

pub struct Connection<'a> {
    send_frames: VecDeque<Vec<u8>>,
    frame_state_validator: connection_frame_state::ConnectionFrameStateValidator,

    hpack_send_context: hpack_context::SendContext<'a>,
    hpack_recv_context: hpack_context::RecvContext<'a>,

    streams: HashMap<framing::StreamId, streaming::Stream>,
    stream_blocker: stream_blocker::StreamBlocker,

    promised_streams_queue: VecDeque<framing::StreamId>,

    connection_shared_state: Rc<RefCell<connection_shared_state::ConnectionSharedState>>,

    highest_remote_initiated_stream_identifier: framing::StreamId,

    send_window: u32,
    receive_window: u32
}

impl<'a> Connection<'a> {
    pub fn new(hpack_send_context: hpack_context::SendContext<'a>, hpack_recv_context: hpack_context::RecvContext<'a>, initial_settings: framing::settings::SettingsFrame) -> Connection<'a> {
        let mut new_con = Connection {
            send_frames: VecDeque::new(),
            frame_state_validator: connection_frame_state::ConnectionFrameStateValidator::new(),
            hpack_send_context: hpack_send_context,
            hpack_recv_context: hpack_recv_context,
            streams: HashMap::new(),
            stream_blocker: stream_blocker::StreamBlocker::new(),
            promised_streams_queue: VecDeque::new(),
            connection_shared_state: Rc::new(RefCell::new(connection_shared_state::ConnectionSharedState::new())),
            highest_remote_initiated_stream_identifier: 0,
            send_window: settings::INITIAL_FLOW_CONTROL_WINDOW_SIZE,
            receive_window: settings::INITIAL_FLOW_CONTROL_WINDOW_SIZE
        };

        // TODO this might queue a go_away, the net code needs to check the startup was okay and close the connection otherwise.
        new_con.apply_settings(initial_settings, false);

        new_con
    }

    // TODO rename to recv.
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

                self.handle_flow_control_for_recv(frame.header.length);
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
                        self.try_unblock_streams();
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

                if settings_frame.is_acknowledge() {
                    // TODO handle ack received
                }
                else {
                    self.apply_settings(settings_frame, true);
                }
            },
            framing::FrameType::GoAway => {
                let go_away_frame = framing::go_away::GoAwayFrame::new(&frame.header, &mut frame.payload.into_iter());

                println!("go away frame received from client {:?}", go_away_frame);

                // TODO handle connection shutdown.
                panic!("will crash; did not expect go away from client");
            },
            framing::FrameType::ResetStream => {
                if frame.header.stream_id == 0x0 {
                    self.push_send_go_away_frame(error::HttpError::ConnectionError(
                        error::ErrorCode::ProtocolError,
                        error::ErrorName::MissingStreamIdentifierOnStreamFrame
                    ));
                    return;
                }

                self.move_to_stream(frame_type, frame, app);
            }
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
                            let promise_stream = streaming::Stream::new_promise(promised_stream_id, self.connection_shared_state.clone(), stream_request);

                            temp_streams.push((promised_stream_id, promise_stream));
                            self.promised_streams_queue.push_front(promised_stream_id);
                        }

                        // TODO duplicated code
                        // Fetch any send frames which have been generated on the stream.
                        let mut is_blocked = self.stream_blocker.is_blocking(promised_stream_id);
                        let stream_frames = stream.fetch_send_frames();
                        // TODO doesn't need to peek any more
                        let mut stream_frame_iter = stream_frames.into_iter().rev();
                        while let Some(frame) = stream_frame_iter.next() {
                            match frame.get_frame_type() {
                                framing::FrameType::Data => {
                                    if is_blocked {
                                        self.stream_blocker.block_frame(promised_stream_id, frame);
                                    }
                                    else {
                                        // TODO which is the correct type? because it is a size, it really should be unsigned.
                                        if frame.get_length() as u32 > self.send_window {
                                            // Must not send, block the stream.
                                            self.stream_blocker.block_frame(promised_stream_id, frame);

                                            is_blocked = true;
                                        }
                                        else {
                                            // The frame should be sent, so update the send window.
                                            self.send_window -= frame.get_length() as u32;
                                            self.send_frames.push_front(
                                                Box::new(frame).compress_frame(promised_stream_id)
                                            );
                                        }
                                    }
                                },
                                framing::FrameType::Headers => {
                                    if is_blocked {
                                        self.stream_blocker.block_frame(promised_stream_id, frame);
                                    }
                                    else {
                                        // Not blocked so just send.
                                        self.send_frames.push_front(
                                            Box::new(frame).compress_frame(promised_stream_id)
                                        );
                                    }
                                },
                                _ => {
                                    // Not a controlled frame, just send.
                                    self.send_frames.push_front(
                                        Box::new(frame).compress_frame(promised_stream_id)
                                    );
                                }
                            }
                        }
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

        let stream_id = frame.header.stream_id;

        let mut temp_streams = Vec::new();
        {
            // Ensure there is always a stream with the current identifier.
            if !self.streams.contains_key(&stream_id) {
                self.streams.insert(
                    stream_id,
                    streaming::Stream::new(stream_id, self.connection_shared_state.clone())
                );

                if stream_id <= self.highest_remote_initiated_stream_identifier {
                    // TODO handle connection shutdown.
                    panic!("Invalid stream identifier sent by client.");
                }
                else {
                    self.highest_remote_initiated_stream_identifier = stream_id;
                }
            }

            let stream = self.streams.get_mut(&stream_id).unwrap();

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
                let promise_stream = streaming::Stream::new_promise(promised_stream_id, self.connection_shared_state.clone(), stream_request);

                temp_streams.push((promised_stream_id, promise_stream));
                self.promised_streams_queue.push_front(promised_stream_id);
            }

            // The below is essentially reconstructing part of a response, starting from the frame which exceeds the 
            // send window up to the end of the response. 
            // It could be made more efficient by keeping the response in a block when fetching it from the stream. However,
            // this impacts the server's ability to multiplex and doesn't allow other flow controlled frame types to be 
            // added in the future.

            // TODO the code below could easily be split out into another function?

            // Fetch any send frames which have been generated on the stream.
            let mut is_blocked = self.stream_blocker.is_blocking(stream_id);
            let stream_frames = stream.fetch_send_frames();
            let mut stream_frame_iter = stream_frames.into_iter().rev();
            while let Some(frame) = stream_frame_iter.next() {
                match frame.get_frame_type() {
                    framing::FrameType::Data => {
                        if is_blocked {
                            self.stream_blocker.block_frame(stream_id, frame);
                        }
                        else {
                            if frame.get_length() as u32 > self.send_window {
                                // Must not send, block the stream.
                                self.stream_blocker.block_frame(stream_id, frame);

                                is_blocked = true;
                            }
                            else {
                                // The frame should be sent, so update the send window.
                                self.send_window -= frame.get_length() as u32;
                                self.send_frames.push_front(
                                    Box::new(frame).compress_frame(stream_id)
                                );
                            }
                        }
                    },
                    framing::FrameType::Headers => {
                        if is_blocked {
                            self.stream_blocker.block_frame(stream_id, frame);
                        }
                        else {
                            // Not blocked so just send.
                            self.send_frames.push_front(
                                Box::new(frame).compress_frame(stream_id)
                            );
                        }
                    },
                    _ => {
                        // Not a controlled frame, just send.
                        self.send_frames.push_front(
                            Box::new(frame).compress_frame(stream_id)
                        );
                    }
                }
            }

            info!("Blocked streams {:?}", self.stream_blocker.get_unblock_priorities());
        }

        while let Some((promised_stream_id, promised_stream)) = temp_streams.pop() {
            self.streams.insert(promised_stream_id, promised_stream);
        }
    }

    // Queues a frame to be sent.
    fn push_send_frame(&mut self, frame: Box<framing::CompressibleHttpFrame>, stream_id: framing::StreamId) {
        log_conn_send_frame!("Pushing frame for send", frame);

        self.send_frames.push_back(
            frame.compress_frame(stream_id)
        );
    }

    fn push_send_go_away_frame(&mut self, http_error: error::HttpError) {
        // TODO handle close connection after this is queued.

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

    fn apply_settings(&mut self, settings_frame: framing::settings::SettingsFrame, send_acknowledge: bool) {
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
                    self.connection_shared_state.borrow_mut().incoming_settings.header_table_size = setting.get_value();

                    // Inform the send context that the max size setting has changed.
                    self.hpack_send_context.inform_max_size_setting_changed(self.connection_shared_state.borrow().incoming_settings.header_table_size);
                },
                &settings::SettingName::SettingsEnablePush => {
                    match setting.get_value() {
                        0 => {
                            // TODO when push promise is disabled remotely then any streams which
                            // are reserved remote need to be reset.
                            self.connection_shared_state.borrow_mut().incoming_settings.enable_push = false;
                        },
                        1 => {
                            // There is nothing to be done when this setting is switched on. The next
                            // time the application wants to push promise it will be enabled.
                            self.connection_shared_state.borrow_mut().incoming_settings.enable_push = true;
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
                            // As soon as there is a fatal error, stop processing and let the connection shut down.
                            return;
                        }
                    }
                },
                &settings::SettingName::SettingsMaxConcurrentStreams => {
                    // TODO need to refuse to open new streams if it would exceed the remote limit (for a server this is just limiting the number of push promises)
                    // TODO need to send reset stream with stream refused if the client exceeds the limit we've set. If the client continues to try to open streams
                    // very quickly while open streams are sill being processed then we can send reset with enhance your calm :)
                    self.connection_shared_state.borrow_mut().incoming_settings.max_concurrent_streams = Some(setting.get_value());
                },
                &settings::SettingName::SettingsInitialWindowSize => {
                    let val = setting.get_value();

                    if val <= settings::MAXIMUM_FLOW_CONTROL_WINDOW_SIZE {
                        self.connection_shared_state.borrow_mut().incoming_settings.initial_window_size = val;

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
                        // As soon as there is a fatal error, stop processing and let the connection shut down.
                        return;
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
                        self.connection_shared_state.borrow_mut().incoming_settings.max_frame_size = val;
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
                        // As soon as there is a fatal error, stop processing and let the connection shut down.
                        return;
                    }
                },
                &settings::SettingName::SettingsMaxHeaderListSize => {
                    // TODO no idea how to handle exceeding this limit on send.
                    self.connection_shared_state.borrow_mut().incoming_settings.max_header_list_size = Some(setting.get_value());
                }
            }
        }

        // TODO break this function down so that the logic isn't required.
        if send_acknowledge {
            let mut settings_acknowledge = framing::settings::SettingsFrameCompressModel::new();
            settings_acknowledge.set_acknowledge();

            // TODO const
            self.push_send_frame(Box::new(settings_acknowledge), 0x0);
        }
    }

    fn handle_flow_control_for_recv(&mut self, size: u32) {
        // Check if the sender was allowed to send a payload this size.
        if size > self.receive_window {
            self.push_send_go_away_frame(error::HttpError::ConnectionError(
                error::ErrorCode::FlowControlError,
                error::ErrorName::ConnectionFlowControlWindowNotRespected
            ));
            return;
        }

        // Update the receive window size.
        self.receive_window -= size;

        let update_amount = flow_control::get_window_update_amount(self.receive_window);
        if update_amount > 0 {
            let window_update_frame = framing::window_update::WindowUpdateFrameCompressModel::new(update_amount);
            self.push_send_frame(Box::new(window_update_frame), 0x0);
        }
    }

    fn try_unblock_streams(&mut self) {
        let mut unblock_priorities = self.stream_blocker.get_unblock_priorities();

        while let Some(stream_id) = unblock_priorities.pop_back() {
            let next_send_size = self.stream_blocker.get_next_send_size(stream_id);

            match next_send_size {
                Some(size) => {
                    if (size as u32) < self.send_window {
                        // TODO group these operations together? they're done in several places.
                        self.send_window -= size as u32;

                        let send_frame = self.stream_blocker.get_next_frame(stream_id).unwrap();
                        self.push_send_frame(send_frame, stream_id);
                    }
                    else {
                        continue;
                    }
                },
                None => {
                    // TODO If fetch size failed then it is possible that the block has been cleared and
                    // can be cleaned up.
                    continue;
                }
            }
        }
    }
}
