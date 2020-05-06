//! Messages of the protocol version 2

mod heartbeat;
mod milestone_request;
mod transaction_broadcast;
mod transaction_request;

/// Version identifier of the messages version 2
pub(crate) const MESSAGES_VERSION_2: u8 = 1 << 1;

pub(crate) use heartbeat::Heartbeat;
pub(crate) use milestone_request::MilestoneRequest;
pub(crate) use transaction_broadcast::TransactionBroadcast;
pub(crate) use transaction_request::TransactionRequest;
