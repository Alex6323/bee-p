mod config;
mod r#static;

pub use config::{
    StaticPeeringConfig,
    StaticPeeringConfigBuilder,
};
pub use r#static::StaticPeerManager;
