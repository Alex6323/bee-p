use common::{logger, Result};

use crate::config::Config;
use crate::state::State;

/// The Bee prototype.
pub struct Bee {
    state: State,
}

impl Bee {
    pub fn from_config(_config: Config) -> Self {
        Self {
            state: State::BootingUp,
        }
    }

    pub fn run(&mut self) -> Result<()> {
        // TEMP: simulate some runtime
        logger::info(&self.state.to_string());
        std::thread::sleep(std::time::Duration::from_millis(1000));
        self.state = State::Running;
        logger::info(&self.state.to_string());
        std::thread::sleep(std::time::Duration::from_millis(8000));
        self.state = State::ShuttingDown;
        logger::info(&self.state.to_string());
        std::thread::sleep(std::time::Duration::from_millis(1000));
        Ok(())
    }
}

#[cfg(test)]
mod should {
    use super::*;
    use crate::config::{Config, Host, Peer};

    #[test]
    fn create_bee_from_config() {
        let _bee = Bee::from_config(Config::builder()
            .with_host(Host::from_address("127.0.0.1:1337"))
            .with_peer(Peer::from_address("127.0.0.1:1338"))
            .try_build()
            .expect("error creating config"));
    }
}