use crate::{Multiaddr, PeerId};

use std::str::FromStr;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct MultiaddrPeerId(Multiaddr, PeerId);

impl MultiaddrPeerId {
    pub fn new(multiaddr: Multiaddr, peer_id: PeerId) -> Self {
        Self(multiaddr, peer_id)
    }

    pub fn multiaddr(&self) -> &Multiaddr {
        &self.0
    }

    pub fn peer_id(&self) -> &PeerId {
        &self.1
    }

    pub fn split(self) -> (Multiaddr, PeerId) {
        (self.0, self.1)
    }
}

// TODO: errro handling
impl FromStr for MultiaddrPeerId {
    type Err = ();

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let sep_index = input.rfind("/p2p/").expect("parse error");
        let (multiaddr, peer_id) = input.split_at(sep_index);

        let multiaddr = Multiaddr::from_str(&multiaddr[..]).expect("from_str");
        let peer_id = PeerId::from_str(&peer_id[5..]).expect("from_str");

        Ok(Self(multiaddr, peer_id))
    }
}
