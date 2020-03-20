use crate::config::Config;
use crate::state::State;

use bee_common::{logger, Result};

use async_std::task;

/// The Bee prototype.
pub struct Bee {
    config: Config,
    state: State,
}

impl Bee {
    /// Creates a node from a config.
    ///
    /// ```
    /// # use crate::config::Config;
    /// # let config = task::block_on(Config::load().await).unwrap();
    /// let mut node = Bee::from_config(config);
    /// ```
    pub fn from_config(config: Config) -> Self {
        Self {
            config,
            state: State::BootingUp,
        }
    }

    /// Runs the event loop of the node.
    pub fn run(&mut self) -> Result<()> {
        logger::info(&self.state.to_string());

        self.init();

        match self.config.peers().get(0) {
            None => {
                logger::warn("No static peers specified in the config. Exiting node.");
            }
            Some(peer) => {
                let host_addr = self.config.host().to_string();
                logger::info(&format!("Host address: {}", host_addr));

                let peer_addr = peer.to_string();
                logger::info(&format!("Peer address: {}", peer_addr));
            }
        }

        Ok(())
    }

    fn init(&mut self) {
        // NOTE: nothing to do here atm, just wait a little so the GUI doesn't update too quickly
        task::block_on(async {
            task::sleep(std::time::Duration::from_millis(1000)).await;
        });

        self.set_state(State::Running);
    }

    pub fn shutdown(mut self) {
        if self.state() != State::Running {
            return;
        }

        self.set_state(State::ShuttingDown);

        task::block_on(async {
            if let Err(e) = self.config().save().await {
                logger::error(&e.to_string());
            } else {
                logger::info("Saved config.");
            }

            // FIX: simulating shutdown
            task::sleep(std::time::Duration::from_millis(1000)).await;
        });
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
        let _bee = Bee::from_config(
            Config::builder()
                .with_host(Host::from_address("127.0.0.1:1337"))
                .with_peer(Peer::from_address("127.0.0.1:1338"))
                .try_build()
                .expect("error creating config"),
        );
    }
}
