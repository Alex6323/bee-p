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
