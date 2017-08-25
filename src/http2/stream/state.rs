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

pub enum StreamState {
    Idle,
    ReservedLocal,
    ReservedRemote,
    Open,
    HalfClosedLocal,
    HalfClosedRemote,
    Closed
}

pub struct StreamStateWrapper<S> {
    state: S
}

// Declare types for each state.
pub struct StateIdle;
pub struct StateReservedLocal;
pub struct StateReservedRemote;
pub struct StateOpen;
pub struct StateHalfClosedLocal;
pub struct StateHalfClosedRemote;
pub struct StateClosed;

impl From<StreamStateWrapper<StateIdle>> for StreamStateWrapper<StateOpen> {
    fn from(_state_wrapper: StreamStateWrapper<StateIdle>) -> StreamStateWrapper<StateOpen> {
        StreamStateWrapper {
            state: StateOpen
        }
    }
}

impl From<StreamStateWrapper<StateIdle>> for StreamStateWrapper<StateReservedLocal> {
    fn from(_state_wrapper: StreamStateWrapper<StateIdle>) -> StreamStateWrapper<StateReservedLocal> {
        StreamStateWrapper {
            state: StateReservedLocal
        }
    }
}

impl From<StreamStateWrapper<StateIdle>> for StreamStateWrapper<StateReservedRemote> {
    fn from(_state_wrapper: StreamStateWrapper<StateIdle>) -> StreamStateWrapper<StateReservedRemote> {
        StreamStateWrapper {
            state: StateReservedRemote
        }
    }
}

impl From<StreamStateWrapper<StateReservedLocal>> for StreamStateWrapper<StateHalfClosedRemote> {
    fn from(_state_wrapper: StreamStateWrapper<StateReservedLocal>) -> StreamStateWrapper<StateHalfClosedRemote> {
        StreamStateWrapper {
            state: StateHalfClosedRemote
        }
    }
}

impl From<StreamStateWrapper<StateReservedLocal>> for StreamStateWrapper<StateClosed> {
    fn from(_state_wrapper: StreamStateWrapper<StateReservedLocal>) -> StreamStateWrapper<StateClosed> {
        StreamStateWrapper {
            state: StateClosed
        }
    }
}

impl From<StreamStateWrapper<StateReservedRemote>> for StreamStateWrapper<StateHalfClosedLocal> {
    fn from(_state_wrapper: StreamStateWrapper<StateReservedRemote>) -> StreamStateWrapper<StateHalfClosedLocal> {
        StreamStateWrapper {
            state: StateHalfClosedLocal
        }
    }
}

impl From<StreamStateWrapper<StateReservedRemote>> for StreamStateWrapper<StateClosed> {
    fn from(_state_wrapper: StreamStateWrapper<StateReservedRemote>) -> StreamStateWrapper<StateClosed> {
        StreamStateWrapper {
            state: StateClosed
        }
    }
}

impl From<StreamStateWrapper<StateOpen>> for StreamStateWrapper<StateHalfClosedRemote> {
    fn from(_state_wrapper: StreamStateWrapper<StateOpen>) -> StreamStateWrapper<StateHalfClosedRemote> {
        StreamStateWrapper {
            state: StateHalfClosedRemote
        }
    }
}

impl From<StreamStateWrapper<StateOpen>> for StreamStateWrapper<StateClosed> {
    fn from(_state_wrapper: StreamStateWrapper<StateOpen>) -> StreamStateWrapper<StateClosed> {
        StreamStateWrapper {
            state: StateClosed
        }
    }
}

impl From<StreamStateWrapper<StateOpen>> for StreamStateWrapper<StateHalfClosedLocal> {
    fn from(_state_wrapper: StreamStateWrapper<StateOpen>) -> StreamStateWrapper<StateHalfClosedLocal> {
        StreamStateWrapper {
            state: StateHalfClosedLocal
        }
    }
}

impl From<StreamStateWrapper<StateHalfClosedRemote>> for StreamStateWrapper<StateClosed> {
    fn from(_state_wrapper: StreamStateWrapper<StateHalfClosedRemote>) -> StreamStateWrapper<StateClosed> {
        StreamStateWrapper {
            state: StateClosed
        }
    }
}
