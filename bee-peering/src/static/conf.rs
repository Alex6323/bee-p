use serde::Deserialize;

// TODO add acceptAnyConnection

const CONF_LIMIT: u8 = 5;
const CONF_PEERS: Vec<String> = Vec::new();

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StaticPeeringConfBuilder {
    pub(crate) limit: Option<u8>,
    pub(crate) peers: Option<Vec<String>>,
}

impl StaticPeeringConfBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn limit(mut self, limit: u8) -> Self {
        self.limit.replace(limit);
        self
    }

    pub fn add_peer(mut self, peer: &str) {
        if self.peers.is_none() {
            self.peers.replace(Vec::new());
        }
        self.peers.unwrap().push(peer.to_owned());
    }

    pub fn build(self) -> StaticPeeringConf {
        StaticPeeringConf {
            limit: self.limit.unwrap_or(CONF_LIMIT),
            peers: self.peers.unwrap_or(CONF_PEERS),
        }
    }
}

#[derive(Clone)]
pub struct StaticPeeringConf {
    pub(crate) limit: u8,
    pub(crate) peers: Vec<String>,
}
