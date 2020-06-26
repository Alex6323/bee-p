//! Autopeering crate

use bee_common::shutdown::Shutdown;

pub mod config;
pub mod events;
pub mod peers;
pub mod salt;

mod discover;

use config::AutopeeringConfig;
use events::EventStream as Events;

pub fn init(config: AutopeeringConfig, shutdown: &mut Shutdown) -> Events {
    todo!()
}
