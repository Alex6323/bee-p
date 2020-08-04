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

use std::net::{IpAddr, Ipv4Addr};

const DEFAULT_BINDING_PORT: u16 = 15600;
const DEFAULT_BINDING_ADDR: IpAddr = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));
pub(crate) const DEFAULT_MAX_TCP_BUFFER_SIZE: usize = 1654;
pub(crate) const DEFAULT_RECONNECT_INTERVAL: u64 = 60;

/// Network configuration builder.
#[derive(Default, Deserialize)]
pub struct NetworkConfigBuilder {
    binding_port: Option<u16>,
    binding_addr: Option<IpAddr>,
    max_tcp_buffer_size: Option<usize>,
    reconnect_interval: Option<u64>,
}

impl NetworkConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn binding_port(mut self, port: u16) -> Self {
        self.binding_port.replace(port);
        self
    }

    pub fn binding_addr(mut self, addr: &str) -> Self {
        match addr.parse() {
            Ok(addr) => {
                self.binding_addr.replace(addr);
            }
            Err(e) => panic!("Error parsing address: {:?}", e),
        }
        self
    }

    pub fn reconnect_interval(mut self, interval: u64) -> Self {
        self.reconnect_interval.replace(interval);
        self
    }

    pub fn max_tcp_buffer_size(mut self, max_tcp_buffer_size: usize) -> Self {
        self.max_tcp_buffer_size.replace(max_tcp_buffer_size);
        self
    }

    /// Builds the network config.
    pub fn finish(self) -> NetworkConfig {
        NetworkConfig {
            binding_port: Port::new(self.binding_port.unwrap_or(DEFAULT_BINDING_PORT)),
            binding_addr: self.binding_addr.unwrap_or(DEFAULT_BINDING_ADDR),
            max_tcp_buffer_size: self.max_tcp_buffer_size.unwrap_or(DEFAULT_MAX_TCP_BUFFER_SIZE),
            reconnect_interval: self.reconnect_interval.unwrap_or(DEFAULT_RECONNECT_INTERVAL),
        }
    }
}

#[derive(Debug)]
pub struct NetworkConfig {
    pub(crate) binding_port: Port,
    pub(crate) binding_addr: IpAddr,
    pub(crate) max_tcp_buffer_size: usize,
    pub(crate) reconnect_interval: u64,
}

impl NetworkConfig {
    pub fn builder() -> NetworkConfigBuilder {
        NetworkConfigBuilder::new()
    }

    pub fn socket_addr(&self) -> Address {
        match self.binding_addr {
            IpAddr::V4(address) => Address::from_v4_addr_and_port(address, self.binding_port),
            IpAddr::V6(address) => Address::from_v6_addr_and_port(address, self.binding_port),
        }
    }
}
