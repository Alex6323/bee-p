use log;
use serde::Deserialize;

pub(crate) const CONF_PATH: &str = "./conf.toml";
const CONF_LOG_LEVEL: &str = "info";

#[derive(Default, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NodeConfBuilder {
    log_level: Option<String>,
}

impl NodeConfBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn log_level(mut self, log_level: &str) -> Self {
        self.log_level.replace(log_level.to_string());
        self
    }

    pub fn build(self) -> NodeConf {
        let log_level = match self.log_level.unwrap_or(CONF_LOG_LEVEL.to_string()).as_str() {
            "trace" => log::LevelFilter::Trace,
            "debug" => log::LevelFilter::Debug,
            "info" => log::LevelFilter::Info,
            "warn" => log::LevelFilter::Warn,
            "error" => log::LevelFilter::Error,
            _ => log::LevelFilter::Info,
        };

        NodeConf { log_level }
    }
}

pub struct NodeConf {
    pub(crate) log_level: log::LevelFilter,
}
