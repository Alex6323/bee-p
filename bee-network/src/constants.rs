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

use std::net::{IpAddr, Ipv4Addr};

// TODO: use Port type instead of primitive
pub(crate) const DEFAULT_BINDING_PORT: u16 = 15600;
pub(crate) const DEFAULT_BINDING_ADDR: IpAddr = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));

pub(crate) const MAX_BUFFER_SIZE: usize = 1654;
pub(crate) const BYTES_CHANNEL_CAPACITY: usize = 10000;
