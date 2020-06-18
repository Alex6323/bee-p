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

pub(crate) const DEFAULT_REST_BINDING_PORT: u16 = 3030;
pub(crate) const DEFAULT_REST_BINDING_ADDR: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

/// API configuration builder.
#[derive(Default, Deserialize)]
pub struct ApiConfigBuilder {
    rest_binding_port: Option<u16>,
    rest_binding_addr: Option<IpAddr>,
}

impl ApiConfigBuilder {
    /// Creates a new config builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the binding port for the REST service.
    pub fn rest_binding_port(mut self, port: u16) -> Self {
        self.rest_binding_port.replace(port);
        self
    }

    /// Sets the binding address for the REST service.
    pub fn rest_binding_addr(mut self, addr: &str) -> Self {
        match addr.parse() {
            Ok(addr) => {
                self.rest_binding_addr.replace(addr);
            }
            Err(e) => panic!("Error parsing address: {:?}", e),
        }
        self
    }

    /// Builds the API config.
    pub fn finish(self) -> ApiConfig {
        ApiConfig {
            rest_binding_port: self.rest_binding_port.unwrap_or(DEFAULT_REST_BINDING_PORT),
            rest_binding_addr: self.rest_binding_addr.unwrap_or(DEFAULT_REST_BINDING_ADDR),
        }
    }
}

/// API configuration.
#[derive(Clone, Copy, Debug)]
pub struct ApiConfig {
    pub(crate) rest_binding_port: u16,
    pub(crate) rest_binding_addr: IpAddr,
}

impl ApiConfig {
    /// Returns a builder for this config.
    pub fn build() -> ApiConfigBuilder {
        ApiConfigBuilder::new()
    }

    /// Returns the listening address.
    pub fn rest_socket_addr(&self) -> SocketAddr {
        match self.rest_binding_addr {
            IpAddr::V4(addr) => SocketAddr::new(IpAddr::V4(addr), self.rest_binding_port),
            IpAddr::V6(addr) => SocketAddr::new(IpAddr::V6(addr), self.rest_binding_port),
        }
    }

    /// Returns the port of the listening address.
    pub fn rest_binding_port(&self) -> u16 {
        self.rest_binding_port
    }

    /// Returns the listening IP address.
    pub fn rest_binding_addr(&self) -> IpAddr {
        self.rest_binding_addr
    }
}
