mod conf;
mod version;

pub(crate) use conf::{
    slice_eq,
    COORDINATOR_BYTES,
    MINIMUM_WEIGHT_MAGNITUDE,
};
pub(crate) use version::{
    supported_version,
    SUPPORTED_VERSIONS,
};

pub use conf::{
    HANDSHAKE_SEND_BOUND,
    HEARTBEAT_SEND_BOUND,
    MILESTONE_REQUEST_SEND_BOUND,
    TRANSACTION_BROADCAST_SEND_BOUND,
    TRANSACTION_REQUEST_SEND_BOUND,
};
