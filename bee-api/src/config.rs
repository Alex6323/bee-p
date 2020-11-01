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

use serde::Deserialize;

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

pub(crate) const DEFAULT_BINDING_PORT: u16 = 14265;
pub(crate) const DEFAULT_BINDING_IP_ADDR: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

/// API configuration builder.
#[derive(Default, Deserialize)]
pub struct ApiConfigBuilder {
    binding_port: Option<u16>,
    binding_ip_addr: Option<IpAddr>,
}

impl ApiConfigBuilder {
    /// Creates a new config builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the binding port for the REST API.
    pub fn binding_port(mut self, port: u16) -> Self {
        self.binding_port.replace(port);
        self
    }

    /// Sets the binding IP address for the REST API.
    pub fn binding_ip_addr(mut self, addr: &str) -> Self {
        match addr.parse() {
            Ok(addr) => {
                self.binding_ip_addr.replace(addr);
            }
            Err(e) => panic!("Error parsing IP address: {:?}", e),
        }
        self
    }

    /// Builds the API config.
    pub fn finish(self) -> ApiConfig {
        let binding_socket_addr = match self.binding_ip_addr.unwrap_or(DEFAULT_BINDING_IP_ADDR) {
            IpAddr::V4(ip) => SocketAddr::new(IpAddr::V4(ip), self.binding_port.unwrap_or(DEFAULT_BINDING_PORT)),
            IpAddr::V6(ip) => SocketAddr::new(IpAddr::V6(ip), self.binding_port.unwrap_or(DEFAULT_BINDING_PORT)),
        };
        ApiConfig { binding_socket_addr }
    }
}

/// API configuration.
#[derive(Clone, Copy, Debug)]
pub struct ApiConfig {
    pub(crate) binding_socket_addr: SocketAddr,
}

impl ApiConfig {
    /// Returns a builder for this config.
    pub fn build() -> ApiConfigBuilder {
        ApiConfigBuilder::new()
    }
    /// Returns the binding address.
    pub fn binding_socket_addr(&self) -> SocketAddr {
        self.binding_socket_addr
    }
}
