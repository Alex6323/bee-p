use std::net::SocketAddr;
use std::net::ToSocketAddrs;

#[derive(Debug, Clone, Copy)]
pub struct Peer(SocketAddr);

impl Peer {
    pub fn from_address(address: impl ToSocketAddrs) -> Self {
        // FIXME
        Self(address.to_socket_addrs().expect("error resolving address").nth(0).expect("error"))
    }
}

impl std::fmt::Display for Peer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

pub struct Peers(Vec<Peer>);

impl Peers {
    pub fn new() -> Self {
        Self(vec![])
    }

    pub fn add(&mut self, peer: Peer) {
        self.0.push(peer);
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Host(SocketAddr);

impl Host {
    pub fn from_address(address: impl ToSocketAddrs) -> Self {
        // FIXME
        Self(address.to_socket_addrs().expect("error resolving address").nth(0).expect("error"))
    }
}

impl std::fmt::Display for Host {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

pub struct ConfigBuilder {
    host: Option<Host>,
    peers: Peers,
}

impl ConfigBuilder {
    pub fn with_host(mut self, host: Host) -> Self {
        self.host.replace(host);
        self
    }

    pub fn with_peer(mut self, peer: Peer) -> Self {
        self.peers.add(peer);
        self
    }

    pub fn try_build(self) -> common::Result<Config> {
        if self.peers.is_empty() {
            return Err(common::Error::ConfigError { key: "peers", msg: "error: you haven't configured any peers" });
        }

        Ok(Config {
            host: self.host.ok_or(common::Error::ConfigError { key: "host", msg: "error: you haven't configured the host address"})?,
            peers: self.peers,
        })
    }
}

pub struct Config {
    host: Host,
    peers: Peers,
}

impl Config {
    pub fn build() -> ConfigBuilder {
        ConfigBuilder {
            host: None,
            peers: Peers::new(),
        }
    }

    pub fn host(&self) -> &Host {
        &self.host
    }

    pub fn peers(&self) -> &Peers {
        &self.peers
    }
}

#[cfg(test)]
mod should {
    use super::*;

    #[test]
    fn create_config_from_builder() {
        let config = Config::build()
            .with_host(Host::from_address("127.0.0.1:1337"))
            .with_peer(Peer::from_address("127.0.0.1:1338"))
            .with_peer(Peer::from_address("127.0.0.1:1339"))
            .try_build()
            .expect("error creating config");

        assert_eq!(config.host().to_string(), "127.0.0.1:1337");
        assert_eq!(config.peers().len(), 2);
    }
}