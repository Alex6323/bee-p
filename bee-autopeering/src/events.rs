use crate::peers::{Peer, PeerId};

use futures::channel::mpsc;

pub type EventStream = mpsc::Sender<Event>;

#[non_exhaustive]
pub enum Event {
    /// Triggered, wehen a new peer has been discovered and verified.
    PeerDiscovered { peer: Peer },

    /// Triggered, wehen a discovered and verified peer could not be re-verified.
    PeerDeleted { peer: PeerId },
}
