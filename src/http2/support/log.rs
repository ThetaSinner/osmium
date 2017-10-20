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

#[macro_export]
macro_rules! log_conn_frame {
    ( $msg:expr, $frame:expr ) => {
        #[cfg(feature = "osmium_support")]
        {
            debug!("(conn) {}: {:?} {:?}", $msg, $frame.header, $frame.payload);
        }
    };
}

#[macro_export]
macro_rules! log_conn_send_frame {
    ( $msg:expr, $frame:expr ) => {
        #[cfg(feature = "osmium_support")]
        {
            debug!("(conn) {}: {:?}", $msg, $frame);
        }
    };
}

#[macro_export]
macro_rules! log_stream_recv {
    ( $msg:expr, $id:expr, $state:expr, $frame:expr ) => {
        #[cfg(feature = "osmium_support")]
        {
            debug!("(stream) {}: {}, {:?} {:?} {:?}", $msg, $id, $state, $frame.header, $frame.payload);
        }
    };
}

#[macro_export]
macro_rules! log_stream_post_recv {
    ( $msg:expr, $id:expr, $state:expr ) => {
        #[cfg(feature = "osmium_support")]
        {
            debug!("(stream) {}: {}, {:?}", $msg, $id, $state);
        }
    };
}

#[macro_export]
macro_rules! log_stream_send_frame {
    ( $msg:expr, $id:expr, $frame:expr ) => {
        #[cfg(feature = "osmium_support")]
        {
            if $frame.get_length() < 100 {
                debug!("(stream) {}: {}, {:?}", $msg, $id, $frame);
            }
            else {
                debug!("(stream) {}: {}, [type {:?}] [flags {:?}] (payload too long to print)", $msg, $id, $frame.get_frame_type(), $frame.get_flags());
            }
        }
    };
}

#[macro_export]
macro_rules! log_compressed_frame {
    ( $msg:expr, $compressed_frame:expr ) => {
        #[cfg(feature = "osmium_support")]
        {
            trace!("{}: {:?}", $msg, $compressed_frame);
        }
    };
}
