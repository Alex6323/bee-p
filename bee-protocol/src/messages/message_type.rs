use crate::messages::errors::MessageError;
use crate::messages::handshake::Handshake;
use crate::messages::heartbeat::Heartbeat;
use crate::messages::legacy_gossip::LegacyGossip;
use crate::messages::message::Message;
use crate::messages::milestone_request::MilestoneRequest;
use crate::messages::transaction_broadcast::TransactionBroadcast;
use crate::messages::transaction_request::TransactionRequest;

use std::ops::Deref;

#[non_exhaustive]
#[derive(Clone)]
pub enum MessageType {
    Handshake(Handshake),
    LegacyGossip(LegacyGossip),
    MilestoneRequest(MilestoneRequest),
    TransactionBroadcast(TransactionBroadcast),
    TransactionRequest(TransactionRequest),
    Heartbeat(Heartbeat),
}

impl Deref for MessageType {
    type Target = dyn Message<Error = MessageError>;

    fn deref<'a>(&'a self) -> &'a Self::Target {
        match self {
            MessageType::Handshake(message) => message,
            MessageType::LegacyGossip(message) => message,
            MessageType::MilestoneRequest(message) => message,
            MessageType::TransactionBroadcast(message) => message,
            MessageType::TransactionRequest(message) => message,
            MessageType::Heartbeat(message) => message,
        }
    }
}
