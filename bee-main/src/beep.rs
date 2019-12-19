use std::collections::HashMap;
use std::net::SocketAddr;

use bee_network::Peer;

/// The Bee prototype.
pub struct Beep {
    peers: HashMap<SocketAddr, Peer>,
}

impl Beep {
    pub fn new() -> Self {
        Beep {
            peers: HashMap::new(),
        }
    }
}
