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

use crate::address::{Address, Port};

use serde::Deserialize;

use std::{net::IpAddr, time::Duration};

/// Network configuration builder.
#[derive(Default, Deserialize)]
pub struct NetworkConfigBuilder {
    binding_port: Option<u16>,
    binding_addr: Option<IpAddr>,
    reconnect_interval: Option<Duration>,
}

impl NetworkConfigBuilder {
    /// Creates a new config builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the binding port for the network.
    pub fn binding_port(mut self, port: u16) -> Self {
        self.binding_port.replace(port);
        self
    }

    /// Sets the binding address for the network.
    pub fn binding_addr(mut self, addr: &str) -> Self {
        match addr.parse() {
            Ok(addr) => {
                self.binding_addr.replace(addr);
            }
            Err(e) => panic!("Error parsing address: {:?}", e),
        }
        self
    }

    /// Sets the interval (in seconds) reconnection attempts occur.
    pub fn reconnect_interval(mut self, interval: Duration) -> Self {
        self.reconnect_interval.replace(interval);
        self
    }

    /// Builds the network config.
    pub fn finish(self) -> NetworkConfig {
        NetworkConfig {
            binding_port: self.binding_port.unwrap_or(crate::constants::DEFAULT_BINDING_PORT),
            binding_addr: self.binding_addr.unwrap_or(crate::constants::DEFAULT_BINDING_ADDR),
            reconnect_interval: self
                .reconnect_interval
                .unwrap_or(crate::constants::DEFAULT_RECONNECT_INTERVAL),
        }
    }
}

/// Network configuration.
#[derive(Clone, Copy, Debug)]
pub struct NetworkConfig {
    // TODO: use Port instead of u16
    pub(crate) binding_port: u16,
    pub(crate) binding_addr: IpAddr,
    pub(crate) reconnect_interval: Duration,
}

impl NetworkConfig {
    /// Returns a builder for this config.
    pub fn build() -> NetworkConfigBuilder {
        NetworkConfigBuilder::new()
    }

    /// Returns the listening address.
    pub fn socket_addr(&self) -> Address {
        match self.binding_addr {
            IpAddr::V4(addr) => Address::from_v4_addr_and_port(addr, Port(self.binding_port)),
            IpAddr::V6(addr) => Address::from_v6_addr_and_port(addr, Port(self.binding_port)),
        }
    }

    /// Returns the port of the listening address.
    pub fn binding_port(&self) -> u16 {
        self.binding_port
    }

    /// Returns the listening IP address.
    pub fn binding_addr(&self) -> IpAddr {
        self.binding_addr
    }

    /// Returns the interval between reconnect attempts.
    pub fn reconnect_interval(&self) -> Duration {
        self.reconnect_interval
    }
}
