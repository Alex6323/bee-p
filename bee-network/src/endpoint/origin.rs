// Copyright 2020 IOTA Stiftung
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with
// the License. You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on
// an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and limitations under the License.

use std::fmt;

/// The connection relationship with an endpoint.
#[derive(Clone, Debug)]
pub enum Origin {
    /// Incoming connection attempt that got accepted.
    Inbound,

    /// Outgoing connection attempt that got accepted.
    Outbound,

    /// Participating endpoints are not bound to eachother.
    Unbound,
}

impl fmt::Display for Origin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Origin::Outbound => write!(f, "outbound"),
            Origin::Inbound => write!(f, "inbound"),
            Origin::Unbound => write!(f, "unbound"),
        }
    }
}
