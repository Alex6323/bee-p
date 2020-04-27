use bee_network::{
    NetworkConfig,
    NetworkConfigBuilder,
};
use bee_peering::{
    PeeringConfig,
    PeeringConfigBuilder,
};
use bee_protocol::{
    ProtocolConfig,
    ProtocolConfigBuilder,
};
use bee_snapshot::{
    SnapshotConfig,
    SnapshotConfigBuilder,
};

use log;
use serde::Deserialize;

pub(crate) const CONFIG_PATH: &str = "./config.toml";
const CONFIG_LOG_LEVEL: &str = "info";

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeConfigBuilder {
    log_level: Option<String>,
    network: NetworkConfigBuilder,
    peering: PeeringConfigBuilder,
    protocol: ProtocolConfigBuilder,
    snapshot: SnapshotConfigBuilder,
}

impl NodeConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn log_level(mut self, log_level: &str) -> Self {
        self.log_level.replace(log_level.to_string());
        self
    }

    pub fn build(self) -> NodeConfig {
        let log_level = match self.log_level.unwrap_or(CONFIG_LOG_LEVEL.to_owned()).as_str() {
            "trace" => log::LevelFilter::Trace,
            "debug" => log::LevelFilter::Debug,
            "info" => log::LevelFilter::Info,
            "warn" => log::LevelFilter::Warn,
            "error" => log::LevelFilter::Error,
            _ => log::LevelFilter::Info,
        };

        NodeConfig {
            log_level,
            network: self.network.build(),
            peering: self.peering.build(),
            protocol: self.protocol.build(),
            snapshot: self.snapshot.build(),
        }
    }
}

pub struct NodeConfig {
    pub(crate) log_level: log::LevelFilter,
    pub(crate) network: NetworkConfig,
    pub(crate) peering: PeeringConfig,
    pub(crate) protocol: ProtocolConfig,
    pub(crate) snapshot: SnapshotConfig,
}
