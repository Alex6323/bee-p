use crate::message::{
    Handshake, Heartbeat, LegacyGossip, MilestoneRequest, TransactionBroadcast, TransactionRequest,
};

use futures::channel::mpsc::{channel, Receiver, Sender};

const HANDSHAKE_QUEUE_SIZE: usize = 1000;
const HEARTBEAT_QUEUE_SIZE: usize = 1000;
const LEGACY_GOSSIP_QUEUE_SIZE: usize = 1000;
const MILESTONE_REQUEST_QUEUE_SIZE: usize = 1000;
const TRANSACTION_BROADCAST_QUEUE_SIZE: usize = 1000;
const TRANSACTION_REQUEST_QUEUE_SIZE: usize = 1000;

pub(crate) struct NeighborQueues {
    pub(crate) handshake: (Sender<Handshake>, Receiver<Handshake>),
    pub(crate) heartbeat: (Sender<Heartbeat>, Receiver<Heartbeat>),
    pub(crate) legacy_gossip: (Sender<LegacyGossip>, Receiver<LegacyGossip>),
    pub(crate) milestone_request: (Sender<MilestoneRequest>, Receiver<MilestoneRequest>),
    pub(crate) transaction_broadcast:
        (Sender<TransactionBroadcast>, Receiver<TransactionBroadcast>),
    pub(crate) transaction_request: (Sender<TransactionRequest>, Receiver<TransactionRequest>),
}

impl Default for NeighborQueues {
    fn default() -> Self {
        Self {
            handshake: channel(HANDSHAKE_QUEUE_SIZE),
            heartbeat: channel(HEARTBEAT_QUEUE_SIZE),
            legacy_gossip: channel(LEGACY_GOSSIP_QUEUE_SIZE),
            milestone_request: channel(MILESTONE_REQUEST_QUEUE_SIZE),
            transaction_broadcast: channel(TRANSACTION_BROADCAST_QUEUE_SIZE),
            transaction_request: channel(TRANSACTION_REQUEST_QUEUE_SIZE),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
}
