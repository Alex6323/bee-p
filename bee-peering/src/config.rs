use crate::r#static::{StaticPeeringConfig, StaticPeeringConfigBuilder};

use serde::Deserialize;

#[derive(Default, Deserialize)]
pub struct PeeringConfigBuilder {
    r#static: StaticPeeringConfigBuilder,
}

impl PeeringConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build(self) -> PeeringConfig {
        PeeringConfig {
            r#static: self.r#static.build(),
        }
    }
}

pub struct PeeringConfig {
    pub r#static: StaticPeeringConfig,
}
