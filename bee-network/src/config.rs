use crate::address::{
    Address,
    Port,
};

use serde::Deserialize;

use std::net::{
    IpAddr,
    Ipv4Addr,
};

const DEFAULT_BINDING_PORT: u16 = 15600;
const DEFAULT_BINDING_ADDR: IpAddr = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));

/// Network configuration.
#[derive(Clone)]
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

/// Network configuration builder.
#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase")]
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
