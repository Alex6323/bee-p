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

use thiserror::Error;

use std::{
    net::{IpAddr, SocketAddr},
    sync::Arc,
};

use dashmap::DashSet;
use tokio::net;

#[derive(Debug, Error)]
pub(crate) enum Error {
    #[error("Address could not be parsed.")]
    AddressParseError(#[from] std::io::Error),

    #[error("Address could not be resolved.")]
    AddressResolveError,
}

pub(crate) async fn resolve_address(address: &str) -> Result<SocketAddr, Error> {
    net::lookup_host(address)
        .await?
        .next()
        .ok_or(Error::AddressResolveError)
}

const INITIAL_IP_FILTER_CAPACITY: usize = 16;

#[derive(Clone, Debug)]
pub(crate) struct IpFilter(Arc<DashSet<IpAddr>>);

impl IpFilter {
    pub fn new() -> Self {
        Self(Arc::new(DashSet::with_capacity(INITIAL_IP_FILTER_CAPACITY)))
    }

    pub fn insert(&self, ip_address: IpAddr) -> bool {
        self.0.insert(ip_address)
    }

    pub fn remove(&self, ip_address: &IpAddr) -> bool {
        self.0.remove(ip_address).is_some()
    }

    pub fn contains(&self, ip_address: &IpAddr) -> bool {
        self.0.contains(&ip_address)
    }

    pub fn clear(&self) {
        self.0.clear();
    }
}
