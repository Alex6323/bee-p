mod errors;
mod handshake;
mod heartbeat;
mod legacy_gossip;
mod message;
mod message_builder;
mod milestone_request;
mod transaction_broadcast;
mod transaction_request;

pub use errors::MessageError;
pub use handshake::Handshake;
pub use heartbeat::Heartbeat;
pub use legacy_gossip::LegacyGossip;
pub use message::Message;
pub use message_builder::read_message;
pub use milestone_request::MilestoneRequest;
pub use transaction_broadcast::TransactionBroadcast;
pub use transaction_request::TransactionRequest;

use std::ops::Deref;

#[non_exhaustive]
pub enum MessageType {
    Handshake(Handshake),
    LegacyGossip(LegacyGossip),
    MilestoneRequest(MilestoneRequest),
    TransactionBroadcast(TransactionBroadcast),
    TransactionRequest(TransactionRequest),
    Heartbeat(Heartbeat),
}

impl Deref for MessageType {
    type Target = dyn Message;

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
