use crate::message::{Handshake, Heartbeat, LegacyGossip, MilestoneRequest, TransactionBroadcast, TransactionRequest};

use futures::channel::mpsc::{channel, Receiver, Sender};

// TODO analyze default values
const HANDSHAKE_CHANNEL_SIZE: usize = 1000;
const LEGACY_GOSSIP_CHANNEL_SIZE: usize = 1000;
const MILESTONE_REQUEST_CHANNEL_SIZE: usize = 1000;
const TRANSACTION_BROADCAST_CHANNEL_SIZE: usize = 1000;
const TRANSACTION_REQUEST_CHANNEL_SIZE: usize = 1000;
const HEARTBEAT_CHANNEL_SIZE: usize = 1000;

pub(crate) struct NeighborSenders {
    // TODO we probably don't need this one
    pub(crate) handshake: Sender<Handshake>,
    pub(crate) legacy_gossip: Sender<LegacyGossip>,
    pub(crate) milestone_request: Sender<MilestoneRequest>,
    pub(crate) transaction_broadcast: Sender<TransactionBroadcast>,
    pub(crate) transaction_request: Sender<TransactionRequest>,
    pub(crate) heartbeat: Sender<Heartbeat>,
}

pub(crate) struct NeighborReceivers {
    // TODO we probably don't need this one
    pub(crate) handshake: Receiver<Handshake>,
    pub(crate) legacy_gossip: Receiver<LegacyGossip>,
    pub(crate) milestone_request: Receiver<MilestoneRequest>,
    pub(crate) transaction_broadcast: Receiver<TransactionBroadcast>,
    pub(crate) transaction_request: Receiver<TransactionRequest>,
    pub(crate) heartbeat: Receiver<Heartbeat>,
}

pub(crate) struct NeighborChannels {
    pub(crate) senders: NeighborSenders,
    pub(crate) receivers: NeighborReceivers,
}

impl NeighborChannels {
    pub fn new() -> Self {
        let handshake = channel(HANDSHAKE_CHANNEL_SIZE);
        let legacy_gossip = channel(LEGACY_GOSSIP_CHANNEL_SIZE);
        let milestone_request = channel(MILESTONE_REQUEST_CHANNEL_SIZE);
        let transaction_broadcast = channel(TRANSACTION_BROADCAST_CHANNEL_SIZE);
        let transaction_request = channel(TRANSACTION_REQUEST_CHANNEL_SIZE);
        let heartbeat = channel(HEARTBEAT_CHANNEL_SIZE);

        Self {
            senders: NeighborSenders {
                handshake: handshake.0,
                legacy_gossip: legacy_gossip.0,
                milestone_request: milestone_request.0,
                transaction_broadcast: transaction_broadcast.0,
                transaction_request: transaction_request.0,
                heartbeat: heartbeat.0,
            },
            receivers: NeighborReceivers {
                handshake: handshake.1,
                legacy_gossip: legacy_gossip.1,
                milestone_request: milestone_request.1,
                transaction_broadcast: transaction_broadcast.1,
                transaction_request: transaction_request.1,
                heartbeat: heartbeat.1,
            },
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
}
