mod bee;
mod config;
mod screen;
mod state;

pub use crate::bee::Bee;
pub use crate::config::{Config, Host, Peer};

use common::logger;

fn main() {
    logger::init(log::LevelFilter::Info);
    screen::init();

    logger::warn("This node will destroy itself in about 10 seconds.");

    let mut bee = Bee::from_config(Config::builder()
        .with_host(Host::from_address("127.0.0.1:1337"))
        .with_peer(Peer::from_address("127.0.0.1:1338"))
        .try_build()
        .expect("error creating config"));

    assert!(bee.run().is_ok());

    screen::exit();
}
