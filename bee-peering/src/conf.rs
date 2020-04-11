use crate::r#static::{
    StaticPeeringConf,
    StaticPeeringConfBuilder,
};

use serde::Deserialize;

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PeeringConfBuilder {
    r#static: StaticPeeringConfBuilder,
}

impl PeeringConfBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build(self) -> PeeringConf {
        PeeringConf {
            r#static: self.r#static.build(),
        }
    }
}

pub struct PeeringConf {
    pub r#static: StaticPeeringConf,
}
