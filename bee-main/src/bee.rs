use common::Result;

use crate::config::Config;

/// The Bee prototype.
pub struct Bee {
}

impl Bee {
    pub fn from_config(_config: Config) -> Self {
        Self {
        }
    }

    pub fn run(&mut self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod should {
    use super::*;
    use crate::config::{Config, Host, Peer};

    #[test]
    fn create_bee_from_config() {
        let _bee = Bee::from_config(Config::build()
            .with_host(Host::from_address("127.0.0.1:1337"))
            .with_peer(Peer::from_address("127.0.0.1:1338"))
            .try_build()
            .expect("error creating config"));
    }
}