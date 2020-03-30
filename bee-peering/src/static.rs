use crate::PeerManager;

use bee_network::{
    Command::AddEndpoint,
    Network,
    Url,
};

use async_std::task::block_on;
use async_trait::async_trait;
use log::error;

// Manages a peer list and watches a conf file for changes
// Sends changes (peer added/removed) to the network

pub struct StaticPeerManager {
    network: Network,
}

impl StaticPeerManager {
    pub fn new(network: Network) -> Self {
        Self { network: network }
    }

    async fn add_endpoint(&mut self, endpoint: &str) {
        if let Err(e) = self
            .network
            .send(AddEndpoint {
                // TODO handle error / unwrap
                url: block_on(Url::from_str_with_port(endpoint)).unwrap(),
                responder: None,
            })
            .await
        {
            error!("[StaticPeerManager ] Failed to add endpoint {}: {}", endpoint, e);
        }
    }
}

#[async_trait]
impl PeerManager for StaticPeerManager {
    async fn run(mut self) {
        // TODO conf file watcher
        self.add_endpoint("tcp://::1:15600").await;
    }
}
