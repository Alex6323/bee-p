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

use crate::{event::TpsMetricsUpdated, protocol::Protocol, worker::Worker};

use bee_common::{shutdown_stream::ShutdownStream, worker::Error as WorkerError};

use async_std::stream::{interval, Interval};
use async_trait::async_trait;
use futures::{stream::Fuse, StreamExt};
use log::info;

use std::{any::TypeId, time::Duration};

#[derive(Default)]
pub(crate) struct TpsWorker {
    incoming: u64,
    new: u64,
    known: u64,
    stale: u64,
    invalid: u64,
    outgoing: u64,
}

#[async_trait]
impl Worker for TpsWorker {
    const DEPS: &'static [TypeId] = &[];

    type Event = ();
    type Receiver = ShutdownStream<Fuse<Interval>>;

    async fn run(mut self, mut receiver: Self::Receiver) -> Result<(), WorkerError> {
        info!("Running.");

        while receiver.next().await.is_some() {
            self.tps();
        }

        info!("Stopped.");

        Ok(())
    }
}

impl TpsWorker {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn interval() -> Interval {
        interval(Duration::from_secs(1))
    }

    fn tps(&mut self) {
        let incoming = Protocol::get().metrics.transactions_received();
        let new = Protocol::get().metrics.new_transactions();
        let known = Protocol::get().metrics.known_transactions();
        let stale = Protocol::get().metrics.stale_transactions();
        let invalid = Protocol::get().metrics.invalid_transactions();
        let outgoing = Protocol::get().metrics.transactions_sent();

        Protocol::get().bus.dispatch(TpsMetricsUpdated {
            incoming: incoming - self.incoming,
            new: new - self.new,
            known: known - self.known,
            stale: stale - self.stale,
            invalid: invalid - self.invalid,
            outgoing: outgoing - self.outgoing,
        });

        self.incoming = incoming;
        self.new = new;
        self.known = known;
        self.stale = stale;
        self.invalid = invalid;
        self.outgoing = outgoing;
    }
}
