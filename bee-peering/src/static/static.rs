use crate::{
    r#static::config::StaticPeeringConfig,
    PeerManager,
};

use bee_network::{
    Command::AddEndpoint,
    Network,
    Url,
};

use async_std::task::block_on;
use async_trait::async_trait;
use log::warn;

// Manages a peer list and watches a config file for changes
// Sends changes (peer added/removed) to the network

pub struct StaticPeerManager {
    config: StaticPeeringConfig,
    network: Network,
}

impl StaticPeerManager {
    pub fn new(config: StaticPeeringConfig, network: Network) -> Self {
        Self { config, network }
    }

    async fn add_endpoint(&mut self, url: &str) {
        // TODO block ?
        match block_on(Url::from_url_str(url)) {
            Ok(url) => {
                if let Err(e) = self.network.send(AddEndpoint { url, responder: None }).await {
                    warn!("[StaticPeerManager ] Failed to add endpoint \"{}\": {}", url, e);
                }
            }
            Err(e) => {
                warn!("[StaticPeerManager ] Failed to resolve URL \"{}\": {}", url, e);
            }
        }
    }
}

#[async_trait]
impl PeerManager for StaticPeerManager {
    async fn run(mut self) {
        // TODO config file watcher
        //TODO use limit
        for peer in self.config.peers.clone() {
            self.add_endpoint(&peer).await;
        }
    }
}
