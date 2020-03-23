use crate::PeerManager;

use bee_network::Command::AddEndpoint;
use bee_network::{
    Network,
    Url,
};

use async_std::task::block_on;
use async_trait::async_trait;

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
        self.network
            .send(AddEndpoint {
                // TODO handle error
                url: block_on(Url::from_str_with_port(endpoint)).unwrap(),
                responder: None,
            })
            .await;
    }
}

#[async_trait]
impl PeerManager for StaticPeerManager {
    async fn run(mut self) {
        // TODO conf file watcher
        self.add_endpoint("tcp://127.0.0.1:15600").await;
    }
}
