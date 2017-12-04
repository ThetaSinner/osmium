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
use std::mem;

// futures
use futures::sync::mpsc as futures_mpsc;

pub struct ShutdownSignaller {
    shutdown_read_tx: Option<futures_mpsc::Sender<u8>>
}

impl ShutdownSignaller{
    pub fn new(shutdown_read_tx: futures_mpsc::Sender<u8>) -> Self {
        ShutdownSignaller {
            shutdown_read_tx: Some(shutdown_read_tx)
        }
    }

    // This has been done quickly to check the net code compiles. It needs to be made way more robust.
    pub fn signal_shutdown(&mut self) {
        if self.shutdown_read_tx.is_some() {
            let mut srtx = None;
            mem::swap(&mut self.shutdown_read_tx, &mut srtx);
            match srtx.unwrap().try_send(1) {
                Ok(_) => {
                    trace!("Signal shutdown read loop");
                },
                Err(e) => {
                    debug!("Attempted read loop shutdown but the signal failed to send, the loop may have already shut down {:?}", e);
                }
            }
        }
        else {
            panic!("cannot call signal shutdown more than once.");
        }
    }
}
