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

use crate::PeerId;

use dashmap::DashSet;

use std::{hash::Hash, net::IpAddr, sync::Arc};

const DEFAULT_BANNED_PEER_CAPACITY: usize = 64;
const DEFAULT_BANNED_ADDR_CAPACITY: usize = 32;

pub type BannedPeerList = BannedList<PeerId>;
pub type BannedAddrList = BannedList<IpAddr>;

#[derive(Clone, Default)]
pub struct BannedList<T: Hash + Eq>(Arc<DashSet<T>>);

impl BannedList<PeerId> {
    pub fn new() -> Self {
        Self(Arc::new(DashSet::with_capacity(DEFAULT_BANNED_PEER_CAPACITY)))
    }
}

impl BannedList<IpAddr> {
    pub fn new() -> Self {
        Self(Arc::new(DashSet::with_capacity(DEFAULT_BANNED_ADDR_CAPACITY)))
    }
}

impl<T> BannedList<T>
where
    T: Hash + Eq,
{
    pub fn insert(&self, value: T) -> bool {
        self.0.insert(value)
    }

    pub fn contains(&self, value: &T) -> bool {
        self.0.contains(value)
    }

    pub fn remove(&self, value: &T) -> bool {
        self.0.remove(value).is_some()
    }

    #[cfg(test)]
    pub fn count(&self) -> usize {
        self.0.len()
    }
}
