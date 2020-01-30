mod bee;
mod config;

pub use crate::bee::Bee;
pub use crate::config::{Config, Host, Peer};

use common::constants::{DEBUG, ENV_VAR};

use std::env;

fn main() {
    env::set_var(ENV_VAR, DEBUG);

    let config = Config::build()
        .with_host(Host::from_address("127.0.0.1:1337"))
        .with_peer(Peer::from_address("127.0.0.1:1338"))
        .with_peer(Peer::from_address("127.0.0.1:1339"))
        .try_build()
        .expect("error creating config");

    let mut bee = Bee::from_config(config);

    assert!(bee.run().is_ok());

    env::remove_var(ENV_VAR);
}
