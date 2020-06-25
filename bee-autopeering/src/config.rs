use serde::Deserialize;

#[derive(Clone, Copy, Debug)]
pub struct AutopeeringConfig {}

impl AutopeeringConfig {
    /// Returns a builder for this config.
    pub fn build() -> AutopeeringConfigBuilder {
        AutopeeringConfigBuilder::new()
    }
}
#[derive(Default, Deserialize)]
pub struct AutopeeringConfigBuilder {}

impl AutopeeringConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn finish(self) -> AutopeeringConfig {
        AutopeeringConfig {}
    }
}
