use crate::PeerManager;

use netzwerk::Command::AddPeer;
use netzwerk::{Network, Peer, Url};

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

    async fn add_peer(&mut self, peer: Peer) {
        self.network
            .send(AddPeer {
                peer: peer,
                connect_attempts: Some(0),
            })
            .await;
    }
}

#[async_trait]
impl PeerManager for StaticPeerManager {
    async fn run(mut self) {
        // TODO conf file watcher
        self.add_peer(Peer::from_url(Url::from_str("tcp://127.0.0.1:15600")))
            .await;
    }
}
