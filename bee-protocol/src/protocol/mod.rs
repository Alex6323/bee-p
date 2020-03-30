mod conf;
mod protocol;
mod version;

// TODO use conf object and do not expose these default values
pub(crate) use conf::{
    slice_eq,
    COORDINATOR_BYTES,
    HEARTBEAT_SEND_BOUND,
    MILESTONE_REQUEST_SEND_BOUND,
    MINIMUM_WEIGHT_MAGNITUDE,
    TRANSACTION_BROADCAST_SEND_BOUND,
    TRANSACTION_REQUEST_SEND_BOUND,
};
pub use protocol::Protocol;
pub(crate) use version::{
    supported_version,
    SUPPORTED_VERSIONS,
};
