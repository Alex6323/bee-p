mod errors;
mod handshake;
mod header;
mod heartbeat;
mod legacy_gossip;
mod message;
mod message_builder;
mod milestone_request;
mod transaction_broadcast;
mod transaction_request;

pub use errors::MessageError;
pub use handshake::Handshake;
pub use header::Header;
pub use heartbeat::Heartbeat;
pub use legacy_gossip::LegacyGossip;
pub use message::Message;
pub use milestone_request::MilestoneRequest;
pub use transaction_broadcast::TransactionBroadcast;
pub use transaction_request::TransactionRequest;

pub enum MessageType {
    Header(Header),
    Handshake(Handshake),
    LegacyGossip(LegacyGossip),
    MilestoneRequest(MilestoneRequest),
    TransactionBroadcast(TransactionBroadcast),
    TransactionRequest(TransactionRequest),
    Heartbeat(Heartbeat),
}
