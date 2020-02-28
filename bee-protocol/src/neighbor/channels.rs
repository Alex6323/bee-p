use crate::message::{
    Handshake, Heartbeat, LegacyGossip, MilestoneRequest, TransactionBroadcast, TransactionRequest,
};

use futures::channel::mpsc::{channel, Receiver, Sender};

// TODO analyze default values
const HANDSHAKE_CHANNEL_SIZE: usize = 1000;
const LEGACY_GOSSIP_CHANNEL_SIZE: usize = 1000;
const MILESTONE_REQUEST_CHANNEL_SIZE: usize = 1000;
const TRANSACTION_BROADCAST_CHANNEL_SIZE: usize = 1000;
const TRANSACTION_REQUEST_CHANNEL_SIZE: usize = 1000;
const HEARTBEAT_CHANNEL_SIZE: usize = 1000;

pub(crate) struct NeighborChannels {
    // TODO we probably don't need this one
    pub(crate) handshake: (Sender<Handshake>, Receiver<Handshake>),
    pub(crate) legacy_gossip: (Sender<LegacyGossip>, Receiver<LegacyGossip>),
    pub(crate) milestone_request: (Sender<MilestoneRequest>, Receiver<MilestoneRequest>),
    pub(crate) transaction_broadcast:
        (Sender<TransactionBroadcast>, Receiver<TransactionBroadcast>),
    pub(crate) transaction_request: (Sender<TransactionRequest>, Receiver<TransactionRequest>),
    pub(crate) heartbeat: (Sender<Heartbeat>, Receiver<Heartbeat>),
}

// TODO remove when we have config
impl Default for NeighborChannels {
    fn default() -> Self {
        Self {
            handshake: channel(HANDSHAKE_CHANNEL_SIZE),
            legacy_gossip: channel(LEGACY_GOSSIP_CHANNEL_SIZE),
            milestone_request: channel(MILESTONE_REQUEST_CHANNEL_SIZE),
            transaction_broadcast: channel(TRANSACTION_BROADCAST_CHANNEL_SIZE),
            transaction_request: channel(TRANSACTION_REQUEST_CHANNEL_SIZE),
            heartbeat: channel(HEARTBEAT_CHANNEL_SIZE),
        }
    }
}

// TODO pass config
impl NeighborChannels {
    fn new() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {

    use super::*;
}
