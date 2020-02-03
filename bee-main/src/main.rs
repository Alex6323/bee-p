mod bee;
mod config;
mod screen;
mod logger;

use logger::BeeLogger;
use screen::BeeScreen;

pub use crate::bee::Bee;
pub use crate::config::{Config, Host, Peer};

use log::Level;

fn main() {
    let logger = BeeLogger::new(Level::Trace);
    logger.init();

    let screen = BeeScreen::new();
    screen.init();

    logger.trace("Just built.");
    logger.info("Starting Bee.");
    logger.error("Not implemented");
    logger.info("This screen will will be displayed for about 10 seconds.");

    let mut bee = Bee::from_config(Config::builder()
        .with_host(Host::from_address("127.0.0.1:1337"))
        .with_peer(Peer::from_address("127.0.0.1:1338"))
        .try_build()
        .expect("error creating config"));

    assert!(bee.run().is_ok());

    screen.exit();
    logger.exit();
}
