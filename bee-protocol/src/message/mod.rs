mod errors;
mod handshake;
mod heartbeat;
mod legacy_gossip;
mod message;
mod milestone_request;
mod transaction_broadcast;
mod transaction_request;

pub(crate) use errors::MessageError;
pub(crate) use handshake::Handshake;
pub(crate) use heartbeat::Heartbeat;
pub(crate) use legacy_gossip::LegacyGossip;
pub(crate) use message::Message;
pub(crate) use milestone_request::MilestoneRequest;
pub(crate) use transaction_broadcast::TransactionBroadcast;
pub(crate) use transaction_request::TransactionRequest;

// TODO Move with Milestone declaration
pub(crate) type MilestoneIndex = u32;
