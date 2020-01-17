use common::Result;

use network::{Peer, PeerId};

use std::collections::HashMap;
use std::net::SocketAddr;

/// The Bee prototype.
pub struct Prototype {
    peers: HashMap<PeerId, Peer>,
}

impl Prototype {
    pub fn from_config(filepath: &str) -> Self {
        Prototype {
            peers: HashMap::new(),
        }
    }

    pub fn run(&mut self) -> Result<()> {
        Ok(())
    }
}
