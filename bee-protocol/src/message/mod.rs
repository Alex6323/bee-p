mod errors;
mod handshake;
mod header;
mod heartbeat;
mod legacy_gossip;
mod message;
mod milestone_request;
mod transaction_broadcast;
mod transaction_request;

pub(crate) use errors::MessageError;
pub use handshake::Handshake;
pub(crate) use header::{
    Header,
    HEADER_SIZE,
    HEADER_TYPE_SIZE,
};
pub use heartbeat::Heartbeat;
pub(crate) use legacy_gossip::LegacyGossip;
pub(crate) use message::Message;
pub use milestone_request::MilestoneRequest;
pub use transaction_broadcast::TransactionBroadcast;
pub use transaction_request::TransactionRequest;

// TODO Move with Milestone declaration
pub(crate) type MilestoneIndex = u32;
