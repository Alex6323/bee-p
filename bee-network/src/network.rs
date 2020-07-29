// Copyright 2020 IOTA Stiftung
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with
// the License. You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on
// an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and limitations under the License.

use crate::{
    commands::{Command, CommandSender},
    config::NetworkConfig,
};

use futures::sink::SinkExt;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Error sending command.")]
    CommandSendFailure(#[from] futures::channel::mpsc::SendError),
}

/// A wrapper around an mpsc channel half, that allows sending `Command`s to the network layer.
///
/// Note, that this type can be cloned to pass it to multiple threads.
#[derive(Clone, Debug)]
pub struct Network {
    config: NetworkConfig,
    inner: CommandSender,
}

impl Network {
    /// Creates a new instance.
    pub fn new(config: NetworkConfig, cmd_sender: CommandSender) -> Self {
        Self {
            config,
            inner: cmd_sender,
        }
    }

    /// Sends a `Command` to the network layer.
    pub async fn send(&mut self, command: Command) -> Result<(), Error> {
        Ok(self.inner.send(command).await?)
    }

    /// Provides access to the network config.
    pub fn config(&self) -> &NetworkConfig {
        &self.config
    }
}
