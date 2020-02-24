use crate::message::Heartbeat;
use crate::message::LegacyGossip;
use crate::message::MilestoneRequest;
use crate::message::TransactionBroadcast;
use crate::message::TransactionRequest;

use crossbeam_channel::{bounded, Receiver, Sender};

const HEARTBEAT_QUEUE_SIZE: usize = 1000;
const LEGACY_GOSSIP_QUEUE_SIZE: usize = 1000;
const MILESTONE_REQUEST_QUEUE_SIZE: usize = 1000;
const TRANSACTION_BROADCAST_QUEUE_SIZE: usize = 1000;
const TRANSACTION_REQUEST_QUEUE_SIZE: usize = 1000;

pub(crate) struct NeighborQueues {
    heartbeat: (Sender<Heartbeat>, Receiver<Heartbeat>),
    legacy_gossip: (Sender<LegacyGossip>, Receiver<LegacyGossip>),
    milestone_request: (Sender<MilestoneRequest>, Receiver<MilestoneRequest>),
    transaction_broadcast: (Sender<TransactionBroadcast>, Receiver<TransactionBroadcast>),
    transaction_request: (Sender<TransactionRequest>, Receiver<TransactionRequest>),
}

impl Default for NeighborQueues {
    fn default() -> Self {
        Self {
            heartbeat: bounded(HEARTBEAT_QUEUE_SIZE),
            legacy_gossip: bounded(LEGACY_GOSSIP_QUEUE_SIZE),
            milestone_request: bounded(MILESTONE_REQUEST_QUEUE_SIZE),
            transaction_broadcast: bounded(TRANSACTION_BROADCAST_QUEUE_SIZE),
            transaction_request: bounded(TRANSACTION_REQUEST_QUEUE_SIZE),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
}
