//! Autopeering crate

use bee_common::shutdown::Shutdown;

pub mod config;
pub mod salt;

use config::AutopeeringConfig;

pub fn init(config: AutopeeringConfig, shutdown: &mut Shutdown) {
    todo!()
}
