use bee_pow::{Cores, Difficulty};

use async_std::fs::File;
use async_std::prelude::*;

use std::fmt;
use std::net::SocketAddr;
use std::net::ToSocketAddrs;

use serde::{Deserialize, Serialize};

use crate::constants::CONFIG_PATH;
use crate::errors::{Error as MainError, Result as MainResult};

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct Peer(SocketAddr);

impl Peer {
    pub fn from_address(address: impl ToSocketAddrs) -> Self {
        // FIXME
        Self(
            address
                .to_socket_addrs()
                .expect("error resolving address")
                .nth(0)
                .expect("error"),
        )
    }
}

impl fmt::Display for Peer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Deserialize, Serialize)]
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

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct Host(SocketAddr);

impl Host {
    pub fn from_address(address: impl ToSocketAddrs) -> Self {
        // FIXME
        Self(
            address
                .to_socket_addrs()
                .expect("error resolving address")
                .nth(0)
                .expect("error"),
        )
    }
}

impl fmt::Display for Host {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

pub struct ConfigBuilder {
    host: Option<Host>,
    peers: Peers,
    pow_difficulty: Option<Difficulty>,
    pow_cores: Option<Cores>,
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

    pub fn with_pow_difficulty(mut self, difficulty: Difficulty) -> Self {
        self.pow_difficulty.replace(difficulty);
        self
    }

    pub fn with_pow_cores(mut self, cores: Cores) -> Self {
        self.pow_cores.replace(cores);
        self
    }

    pub fn try_build(self) -> bee_common::Result<Config> {
        if self.peers.is_empty() {
            return Err(bee_common::Errors::ConfigError {
                key: "peers",
                msg: "error: you haven't configured any peers",
            });
        }

        Ok(Config {
            host: self.host.ok_or(bee_common::Errors::ConfigError {
                key: "host",
                msg: "error: you haven't configured the host address",
            })?,
            peers: self.peers,
            pow_difficulty: self.pow_difficulty.unwrap_or(Difficulty::mainnet()),
            pow_cores: self.pow_cores.unwrap_or(Cores::max()),
        })
    }
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    host: Host,
    peers: Peers,
    pow_difficulty: Difficulty,
    pow_cores: Cores,
}

impl Config {
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder {
            host: None,
            peers: Peers::new(),
            pow_difficulty: Some(Difficulty::mainnet()),
            pow_cores: Some(Cores::max()),
        }
    }

    pub async fn load() -> MainResult<Self> {
        match File::open(CONFIG_PATH).await {
            Ok(mut config_file) => {
                let mut json = String::new();
                match config_file.read_to_string(&mut json).await {
                    Ok(_) => {
                        match serde_json::from_str::<Config>(&json) {
                            Ok(config) => Ok(config),
                            Err(json_err) => Err(MainError::ConfigFromJsonError(json_err)),
                        }
                    },
                    Err(io_err) => Err(MainError::ConfigLoadError(io_err))
                }
            },
            Err(io_err) => Err(MainError::ConfigLoadError(io_err))
        }
    }

    pub async fn save(&self) -> MainResult<()> {
        match serde_json::to_string_pretty(self) {
            Ok(json) => {
                match File::create(CONFIG_PATH).await {
                    Ok(mut file) => {
                        match file.write_all(json.as_bytes()).await {
                            Ok(_) => Ok(()),
                            Err(io_err) => Err(MainError::ConfigSaveError(io_err)),
                        }
                    },
                    Err(io_err) => Err(MainError::ConfigSaveError(io_err)),
                }
            },
            Err(json_err) => Err(MainError::ConfigToJsonError(json_err))
        }
    }

    pub fn host(&self) -> &Host {
        &self.host
    }

    pub fn peers(&self) -> &Peers {
        &self.peers
    }

    pub fn pow_difficulty(&self) -> &Difficulty {
        &self.pow_difficulty
    }

    pub fn pow_cores(&self) -> &Cores {
        &self.pow_cores
    }
}

#[cfg(test)]
mod should {
    use super::*;

    #[test]
    fn create_config_from_builder() {
        let config = Config::builder()
            .with_host(Host::from_address("127.0.0.1:1337"))
            .with_peer(Peer::from_address("127.0.0.1:1338"))
            .with_peer(Peer::from_address("127.0.0.1:1339"))
            .try_build()
            .expect("error creating config");

        assert_eq!(config.host().to_string(), "127.0.0.1:1337");
        assert_eq!(config.peers().len(), 2);

        // Use 'cargo t -- --nocapture' to print the JSON
        let s = serde_json::to_string_pretty(&config).expect("error serializing to JSON");
        println!("JSON:\n{}", s);
    }
}
