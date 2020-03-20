mod conf;
mod version;

pub(crate) use conf::{
    COORDINATOR_BYTES,
    MINIMUM_WEIGHT_MAGNITUDE,
};
pub(crate) use version::{
    supported_version,
    SUPPORTED_VERSIONS,
};
