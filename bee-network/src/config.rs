// Copyright 2020 IOTA Stiftung
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::address::{Address, Port};

use serde::Deserialize;

use std::net::{IpAddr, Ipv4Addr};

const DEFAULT_BINDING_PORT: u16 = 15600;
const DEFAULT_BINDING_ADDR: IpAddr = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));

/// Network configuration builder.
#[derive(Default, Deserialize)]
pub struct NetworkConfigBuilder {
    binding_port: Option<u16>,
    binding_addr: Option<IpAddr>,
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

    /// Builds the network config.
    pub fn build(self) -> NetworkConfig {
        NetworkConfig {
            binding_port: self.binding_port.unwrap_or(DEFAULT_BINDING_PORT),
            binding_addr: self.binding_addr.unwrap_or(DEFAULT_BINDING_ADDR),
        }
    }
}

/// Network configuration.
#[derive(Clone, Copy, Debug)]
pub struct NetworkConfig {
    pub(crate) binding_port: u16,
    pub(crate) binding_addr: IpAddr,
}

impl NetworkConfig {
    /// Returns a builder for this config.
    pub fn builder() -> NetworkConfigBuilder {
        NetworkConfigBuilder {
            binding_port: None,
            binding_addr: None,
        }
    }

    pub(crate) fn socket_addr(&self) -> Address {
        match self.binding_addr {
            IpAddr::V4(addr) => Address::from_v4_addr_and_port(addr, Port(self.binding_port)),
            IpAddr::V6(addr) => Address::from_v6_addr_and_port(addr, Port(self.binding_port)),
        }
    }
}
