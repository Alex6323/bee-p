mod conf;
mod r#static;

pub use conf::{
    StaticPeeringConf,
    StaticPeeringConfBuilder,
};
pub use r#static::StaticPeerManager;
