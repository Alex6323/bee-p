mod conf;

pub(crate) use conf::{
    slice_eq,
    COORDINATOR_BYTES,
};

pub use conf::{
    ProtocolConf,
    ProtocolConfBuilder,
};
