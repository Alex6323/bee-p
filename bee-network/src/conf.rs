use serde::Deserialize;

const CONF_PORT: u16 = 15600;

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct NetworkServerConfBuilder {
    port: Option<u16>,
}

/// A builder for a network configuration
#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkConfBuilder {
    server: NetworkServerConfBuilder,
}

impl NetworkConfBuilder {
    /// Creates a new builder for a network configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the port of the conf builder
    pub fn port(mut self, port: u16) -> Self {
        self.server.port.replace(port);
        self
    }

    /// Builds the conf
    pub fn build(self) -> NetworkConf {
        NetworkConf {
            server: NetworkServerConf {
                port: self.server.port.unwrap_or(CONF_PORT),
            },
        }
    }
}

#[derive(Clone)]
/// A network server configuration
pub struct NetworkServerConf {
    pub(crate) port: u16,
}

#[derive(Clone)]
/// A network configuration
pub struct NetworkConf {
    pub(crate) server: NetworkServerConf,
}
