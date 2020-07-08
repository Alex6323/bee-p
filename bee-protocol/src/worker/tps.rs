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

use crate::protocol::Protocol;

use bee_common_ext::worker::Error as WorkerError;

use async_std::{future::ready, prelude::*};
use futures::channel::oneshot::Receiver;
use log::info;

use std::time::Duration;

pub(crate) struct TpsWorker {
    incoming: u64,
    new: u64,
    known: u64,
    stale: u64,
    invalid: u64,
    outgoing: u64,
}

impl TpsWorker {
    pub(crate) fn new() -> Self {
        Self {
            incoming: 0,
            new: 0,
            known: 0,
            stale: 0,
            invalid: 0,
            outgoing: 0,
        }
    }

    fn tps(&mut self) {
        let incoming = Protocol::get().metrics.transaction_received();
        let new = Protocol::get().metrics.new_transactions_received();
        let known = Protocol::get().metrics.known_transactions_received();
        let stale = Protocol::get().metrics.stale_transactions_received();
        let invalid = Protocol::get().metrics.invalid_transactions_received();
        let outgoing = Protocol::get().metrics.transaction_sent();

        info!(
            "incoming {} new {} known {} stale {} invalid {} outgoing {}",
            incoming - self.incoming,
            new - self.new,
            known - self.known,
            stale - self.stale,
            invalid - self.invalid,
            outgoing - self.outgoing
        );

        self.incoming = incoming;
        self.new = new;
        self.known = known;
        self.stale = stale;
        self.invalid = invalid;
        self.outgoing = outgoing;
    }

    pub(crate) async fn run(mut self, mut shutdown: Receiver<()>) -> Result<(), WorkerError> {
        info!("Running.");

        loop {
            match ready(Ok(()))
                .delay(Duration::from_millis(1000))
                .race(&mut shutdown)
                .await
            {
                Ok(_) => self.tps(),
                Err(_) => break,
            }
        }

        info!("Stopped.");

        Ok(())
    }
}
