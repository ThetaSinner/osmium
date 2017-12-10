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

use http2::stream as streaming;

#[derive(Debug)]
pub enum StreamStateName {
    Idle(StreamState<StateIdle>),
    ReservedLocal(StreamState<StateReservedLocal>),
    ReservedRemote(StreamState<StateReservedRemote>),
    Open(StreamState<StateOpen>),
    HalfClosedLocal(StreamState<StateHalfClosedLocal>),
    HalfClosedRemote(StreamState<StateHalfClosedRemote>),
    Closed(StreamState<StateClosed>)
}

#[derive(Debug)]
pub struct StreamState<S> {
    // This value is never accessed, but it is required to make the generics in the state
    // machine work.
    // TODO is there a better way to write this code so that this value isn't required.
    #[allow(dead_code)]
    pub state: S
}

impl StreamState<StateIdle> {
    pub fn new() -> StreamState<StateIdle> {
        StreamState {
            state: StateIdle
        }
    }
}

#[derive(Debug)]
pub enum StreamClosedReason {
    StreamEnded,
    ResetLocal,
    ResetRemote
}

#[derive(Debug)]
pub struct StreamClosedInfo {
    pub reason: StreamClosedReason
}

// Declare types for each state.
#[derive(Debug)]
pub struct StateIdle;
#[derive(Debug)]
pub struct StateReservedLocal {
    pub stream_request: streaming::StreamRequest
}
#[derive(Debug)]
pub struct StateReservedRemote;
#[derive(Debug)]
pub struct StateOpen;
#[derive(Debug)]
pub struct StateHalfClosedLocal;
#[derive(Debug)]
pub struct StateHalfClosedRemote;
#[derive(Debug)]
pub struct StateClosed {
    pub info: StreamClosedInfo
}

// Declare valid transitions between states.

impl<'a> From<&'a StreamState<StateIdle>> for StreamState<StateOpen> {
    fn from(_state_wrapper: &StreamState<StateIdle>) -> StreamState<StateOpen> {
        StreamState {
            state: StateOpen
        }
    }
}

impl<'a> From<(&'a StreamState<StateIdle>, streaming::StreamRequest)> for StreamState<StateReservedLocal> {
    fn from((_state_wrapper, stream_request): (&StreamState<StateIdle>, streaming::StreamRequest)) -> StreamState<StateReservedLocal> {
        StreamState {
            state: StateReservedLocal { stream_request }
        }
    }
}

impl<'a> From<&'a StreamState<StateIdle>> for StreamState<StateReservedRemote> {
    fn from(_state_wrapper: &StreamState<StateIdle>) -> StreamState<StateReservedRemote> {
        StreamState {
            state: StateReservedRemote
        }
    }
}

impl<'a> From<&'a StreamState<StateReservedLocal>> for StreamState<StateHalfClosedRemote> {
    fn from(_state_wrapper: &StreamState<StateReservedLocal>) -> StreamState<StateHalfClosedRemote> {
        StreamState {
            state: StateHalfClosedRemote
        }
    }
}

impl<'a> From<(&'a StreamState<StateReservedLocal>, StreamClosedInfo)> for StreamState<StateClosed> {
    fn from((_state_wrapper, info): (&StreamState<StateReservedLocal>, StreamClosedInfo)) -> StreamState<StateClosed> {
        StreamState {
            state: StateClosed { info }
        }
    }
}

impl<'a> From<&'a StreamState<StateReservedRemote>> for StreamState<StateHalfClosedLocal> {
    fn from(_state_wrapper: &StreamState<StateReservedRemote>) -> StreamState<StateHalfClosedLocal> {
        StreamState {
            state: StateHalfClosedLocal
        }
    }
}

impl<'a> From<(&'a StreamState<StateReservedRemote>, StreamClosedInfo)> for StreamState<StateClosed> {
    fn from((_state_wrapper, info): (&StreamState<StateReservedRemote>, StreamClosedInfo)) -> StreamState<StateClosed> {
        StreamState {
            state: StateClosed { info }
        }
    }
}

impl<'a> From<&'a StreamState<StateOpen>> for StreamState<StateHalfClosedRemote> {
    fn from(_state_wrapper: &StreamState<StateOpen>) -> StreamState<StateHalfClosedRemote> {
        StreamState {
            state: StateHalfClosedRemote
        }
    }
}

impl<'a> From<(&'a StreamState<StateOpen>, StreamClosedInfo)> for StreamState<StateClosed> {
    fn from((_state_wrapper, info): (&StreamState<StateOpen>, StreamClosedInfo)) -> StreamState<StateClosed> {
        StreamState {
            state: StateClosed { info }
        }
    }
}

impl<'a> From<&'a StreamState<StateOpen>> for StreamState<StateHalfClosedLocal> {
    fn from(_state_wrapper: &StreamState<StateOpen>) -> StreamState<StateHalfClosedLocal> {
        StreamState {
            state: StateHalfClosedLocal
        }
    }
}

impl<'a> From<(&'a StreamState<StateHalfClosedRemote>, StreamClosedInfo)> for StreamState<StateClosed> {
    fn from((_state_wrapper, info): (&StreamState<StateHalfClosedRemote>, StreamClosedInfo)) -> StreamState<StateClosed> {
        StreamState {
            state: StateClosed { info }
        }
    }
}

impl<'a> From<(&'a StreamState<StateHalfClosedLocal>, StreamClosedInfo)> for StreamState<StateClosed> {
    fn from((_state_wrapper, info): (&StreamState<StateHalfClosedLocal>, StreamClosedInfo)) -> StreamState<StateClosed> {
        StreamState {
            state: StateClosed { info }
        }
    }
}
