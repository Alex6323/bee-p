use crate::messages::errors::ProtocolMessageError;
use crate::messages::handshake::Handshake;
use crate::messages::heartbeat::Heartbeat;
use crate::messages::legacy_gossip::LegacyGossip;
use crate::messages::milestone_request::MilestoneRequest;
use crate::messages::transaction_broadcast::TransactionBroadcast;
use crate::messages::transaction_request::TransactionRequest;

use bee_network::Message;

use std::ops::Deref;

#[non_exhaustive]
#[derive(Clone)]
pub enum ProtocolMessageType {
    Handshake(Handshake),
    LegacyGossip(LegacyGossip),
    MilestoneRequest(MilestoneRequest),
    TransactionBroadcast(TransactionBroadcast),
    TransactionRequest(TransactionRequest),
    Heartbeat(Heartbeat),
}

impl Deref for ProtocolMessageType {
    type Target = dyn Message<Error = ProtocolMessageError>;

    fn deref<'a>(&'a self) -> &'a Self::Target {
        match self {
            ProtocolMessageType::Handshake(message) => message,
            ProtocolMessageType::LegacyGossip(message) => message,
            ProtocolMessageType::MilestoneRequest(message) => message,
            ProtocolMessageType::TransactionBroadcast(message) => message,
            ProtocolMessageType::TransactionRequest(message) => message,
            ProtocolMessageType::Heartbeat(message) => message,
        }
    }
}
