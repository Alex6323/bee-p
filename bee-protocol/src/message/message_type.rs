use crate::message::errors::ProtocolMessageError;
use crate::message::handshake::Handshake;
use crate::message::heartbeat::Heartbeat;
use crate::message::legacy_gossip::LegacyGossip;
use crate::message::milestone_request::MilestoneRequest;
use crate::message::transaction_broadcast::TransactionBroadcast;
use crate::message::transaction_request::TransactionRequest;
use crate::message::Message;

use std::ops::Deref;

// TODO probably not needed anymore
#[non_exhaustive]
#[derive(Clone)]
pub(crate) enum ProtocolMessageType {
    Handshake(Handshake),
    LegacyGossip(LegacyGossip),
    MilestoneRequest(MilestoneRequest),
    TransactionBroadcast(TransactionBroadcast),
    TransactionRequest(TransactionRequest),
    Heartbeat(Heartbeat),
}

// TODO probably not needed anymore
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
