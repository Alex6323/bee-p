use bee_common::{logger, Result};

use crate::config::Config;
use crate::state::State;

use async_std::task;

/// The Bee prototype.
pub struct Bee {
    config: Config,
    state: State,
}

impl Bee {
    pub fn from_config(config: Config) -> Self {
        Self {
            config,
            state: State::BootingUp,
        }
    }

    pub fn run(&mut self) -> Result<()> {
        // TEMP: simulate some runtime
        logger::info(&self.state.to_string());
        task::block_on(async {
            task::sleep(std::time::Duration::from_millis(1000)).await;
        });

        self.set_state(State::Running);
        task::block_on(async {
            task::sleep(std::time::Duration::from_millis(8000)).await;
        });

        self.set_state(State::ShuttingDown);
        task::block_on(async {
            task::sleep(std::time::Duration::from_millis(1000)).await;
        });

        Ok(())
    }

    pub fn shutdown(mut self) {
        if self.state() != State::Running {
            return;
        }

        self.set_state(State::ShuttingDown);

        // send shutdown signal
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn state(&self) -> State {
        self.state
    }

    fn set_state(&mut self, state: State) {
        self.state = state;
        logger::info(&self.state.to_string());
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