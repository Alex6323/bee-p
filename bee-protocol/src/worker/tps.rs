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

use std::time::Duration;

use async_std::{future::ready, prelude::*};
use futures::channel::mpsc::Receiver;
use log::info;

pub(crate) struct TpsWorker {
    incoming: u64,
    outgoing: u64,
}

impl TpsWorker {
    pub(crate) fn new() -> Self {
        Self {
            incoming: 0,
            outgoing: 0,
        }
    }

    fn status(&mut self) {
        let received = Protocol::get().metrics.transaction_broadcast_received();
        let sent = Protocol::get().metrics.transaction_broadcast_sent();

        info!(
            "incoming {} outgoing {}",
            received - self.incoming,
            sent - self.outgoing
        );

        self.incoming = received;
        self.outgoing = sent;
    }

    pub(crate) async fn run(mut self, mut shutdown: Receiver<()>) {
        info!("Running.");

        loop {
            match ready(None)
                .delay(Duration::from_millis(1000))
                .race(shutdown.next())
                .await
            {
                Some(_) => {
                    break;
                }
                None => self.status(),
            }
        }

        info!("Stopped.");
    }
}
