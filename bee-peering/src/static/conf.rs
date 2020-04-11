use serde::Deserialize;

const CONF_PEERS: Vec<String> = Vec::new();

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StaticPeeringConfBuilder {
    pub(crate) peers: Option<Vec<String>>,
}

impl StaticPeeringConfBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_peer(mut self, peer: &str) {
        if self.peers.is_none() {
            self.peers.replace(Vec::new());
        }
        self.peers.unwrap().push(peer.to_owned());
    }

    pub fn build(self) -> StaticPeeringConf {
        StaticPeeringConf {
            peers: self.peers.unwrap_or(CONF_PEERS),
        }
    }
}

#[derive(Debug)]
pub struct StaticPeeringConf {
    pub(crate) peers: Vec<String>,
}
