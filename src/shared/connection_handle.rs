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

// TODO remove stream requests and responses from the system completely. There is no difference
// between http1 and http2 request/response so the shared module should contain a single representation
// of each.
use http2::stream::StreamRequest;
use shared::push_error;

/// Trait to be implemented as part of an http implementation. That need not be the connection
/// representation though, any struct which has access to the data required to implement the trait 
/// may implement it. 
/// In particular, an implementation which does not require promises need not provide more than 
/// enabled false and an unreachable error in the push promise.

pub trait ConnectionHandle {
    /// Query the connection to check for push promise enabled.
    fn is_push_enabled(&self) -> bool;

    /// Try to create a new push promise. 
    /// 
    /// The server may reject the promise because
    /// - The remote settings prevent new promises being created.
    /// - The server knows the promise will be rejected by the client, and therefore
    /// decides to reject it immediately.
    /// 
    /// This method MUST NOT be called if `is_push_enabled` yields false in the same application
    /// processing call.
    // TODO point 2 about promise rejection above is not implemented
    fn push_promise(&mut self, request: StreamRequest) -> Option<push_error::PushError>;
}
