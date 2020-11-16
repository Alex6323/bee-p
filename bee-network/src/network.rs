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
    config::NetworkConfig,
    interaction::commands::{Command, CommandSender},
    Multiaddr, PeerId,
};

use thiserror::Error;

use std::sync::Arc;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Error sending command.")]
    CommandSendFailure,
    #[error("Error sending unbounded command.")]
    CommandSendUnboundedFailure,
}

#[derive(Clone, Debug)]
pub struct Network {
    config: Arc<NetworkConfig>,
    command_sender: CommandSender,
    listen_address: Multiaddr,
    local_id: PeerId,
}

impl Network {
    pub(crate) fn new(
        config: NetworkConfig,
        command_sender: CommandSender,
        listen_address: Multiaddr,
        local_id: PeerId,
    ) -> Self {
        Self {
            config: Arc::new(config),
            command_sender,
            listen_address,
            local_id,
        }
    }

    pub async fn send(&mut self, command: Command) -> Result<(), Error> {
        Ok(self
            .command_sender
            .send_async(command)
            .await
            .map_err(|_| Error::CommandSendFailure)?)
    }

    pub fn unbounded_send(&self, command: Command) -> Result<(), Error> {
        Ok(self
            .command_sender
            .send(command)
            .map_err(|_| Error::CommandSendUnboundedFailure)?)
    }

    pub fn config(&self) -> &NetworkConfig {
        &self.config
    }

    pub fn listen_address(&self) -> &Multiaddr {
        &self.listen_address
    }

    pub fn local_id(&self) -> &PeerId {
        &self.local_id
    }
}
