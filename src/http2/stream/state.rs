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

pub enum StreamStateName {
    Idle(StreamState<StateIdle>),
    ReservedLocal(StreamState<StateReservedLocal>),
    ReservedRemote(StreamState<StateReservedRemote>),
    Open(StreamState<StateOpen>),
    HalfClosedLocal(StreamState<StateHalfClosedLocal>),
    HalfClosedRemote(StreamState<StateHalfClosedRemote>),
    Closed(StreamState<StateClosed>)
}

pub struct StreamState<S> {
    state: S
}

impl StreamState<StateIdle> {
    pub fn new() -> StreamState<StateIdle> {
        StreamState {
            state: StateIdle
        }
    }
}

// Declare types for each state.
pub struct StateIdle;
pub struct StateReservedLocal;
pub struct StateReservedRemote;
pub struct StateOpen;
pub struct StateHalfClosedLocal;
pub struct StateHalfClosedRemote;
pub struct StateClosed;

// Declare valid transitions between states.

impl<'a> From<&'a StreamState<StateIdle>> for StreamState<StateOpen> {
    fn from(_state_wrapper: &StreamState<StateIdle>) -> StreamState<StateOpen> {
        StreamState {
            state: StateOpen
        }
    }
}

impl<'a> From<&'a StreamState<StateIdle>> for StreamState<StateReservedLocal> {
    fn from(_state_wrapper: &StreamState<StateIdle>) -> StreamState<StateReservedLocal> {
        StreamState {
            state: StateReservedLocal
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

impl<'a> From<&'a StreamState<StateReservedLocal>> for StreamState<StateClosed> {
    fn from(_state_wrapper: &StreamState<StateReservedLocal>) -> StreamState<StateClosed> {
        StreamState {
            state: StateClosed
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

impl<'a> From<&'a StreamState<StateReservedRemote>> for StreamState<StateClosed> {
    fn from(_state_wrapper: &StreamState<StateReservedRemote>) -> StreamState<StateClosed> {
        StreamState {
            state: StateClosed
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

impl<'a> From<&'a StreamState<StateOpen>> for StreamState<StateClosed> {
    fn from(_state_wrapper: &StreamState<StateOpen>) -> StreamState<StateClosed> {
        StreamState {
            state: StateClosed
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

impl<'a> From<&'a StreamState<StateHalfClosedRemote>> for StreamState<StateClosed> {
    fn from(_state_wrapper: &StreamState<StateHalfClosedRemote>) -> StreamState<StateClosed> {
        StreamState {
            state: StateClosed
        }
    }
}

impl<'a> From<&'a StreamState<StateHalfClosedLocal>> for StreamState<StateClosed> {
    fn from(_state_wrapper: &StreamState<StateHalfClosedLocal>) -> StreamState<StateClosed> {
        StreamState {
            state: StateClosed
        }
    }
}
