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

use bee_common::worker::Error as WorkerError;
use bee_transaction::bundled::Address;

use futures::{
    channel::{mpsc, oneshot},
    future::FutureExt,
    select,
    stream::StreamExt,
};
use log::{info, warn};

use std::collections::HashMap;

pub enum LedgerStateWorkerEvent {
    ApplyDiff(HashMap<Address, i64>),
    GetBalance(Address, oneshot::Sender<Option<u64>>),
}

pub struct LedgerStateWorker {
    state: HashMap<Address, u64>,
}

impl LedgerStateWorker {
    pub fn new(state: HashMap<Address, u64>) -> Self {
        Self { state }
    }

    fn apply_diff(&mut self, diff: HashMap<Address, i64>) {
        for (key, value) in diff {
            self.state
                .entry(key)
                .and_modify(|balance| {
                    if *balance as i64 + value >= 0 {
                        *balance = (*balance as i64 + value) as u64;
                    } else {
                        warn!("Ignoring conflicting diff.");
                    }
                })
                .or_insert(value as u64);
        }
    }

    fn get_balance(&self, address: Address, sender: oneshot::Sender<Option<u64>>) {
        if let Err(e) = sender.send(self.state.get(&address).cloned()) {
            warn!("Failed to send balance: {:?}.", e);
        }
    }

    pub async fn run(
        mut self,
        receiver: mpsc::Receiver<LedgerStateWorkerEvent>,
        shutdown: oneshot::Receiver<()>,
    ) -> Result<(), WorkerError> {
        info!("Running.");

        let mut receiver_fused = receiver.fuse();
        let mut shutdown_fused = shutdown.fuse();

        loop {
            select! {
                _ = shutdown_fused => break,
                event = receiver_fused.next() => {
                    if let Some(event) = event {
                        match event {
                            LedgerStateWorkerEvent::ApplyDiff(diff) => self.apply_diff(diff),
                            LedgerStateWorkerEvent::GetBalance(address, sender) => self.get_balance(address, sender)
                        }
                    }
                }
            }
        }

        info!("Stopped.");

        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    use bee_test::field::rand_trits_field;

    use async_std::task::{block_on, spawn};
    use futures::sink::SinkExt;
    use rand::Rng;

    #[test]
    fn get_balances() {
        let mut rng = rand::thread_rng();
        let mut state = HashMap::new();
        let (mut tx, rx) = mpsc::channel(100);
        let (_shutdown_tx, shutdown_rx) = oneshot::channel();

        for _ in 0..100 {
            state.insert(rand_trits_field::<Address>(), rng.gen_range(0, 100_000_000));
        }

        spawn(LedgerStateWorker::new(state.clone()).run(rx, shutdown_rx));

        for (address, balance) in state {
            let (get_balance_tx, get_balance_rx) = oneshot::channel();
            block_on(tx.send(LedgerStateWorkerEvent::GetBalance(address, get_balance_tx))).unwrap();
            let value = block_on(get_balance_rx).unwrap().unwrap();
            assert_eq!(balance, value)
        }
    }

    #[test]
    fn get_balances_not_found() {
        let mut rng = rand::thread_rng();
        let mut state = HashMap::new();
        let (mut tx, rx) = mpsc::channel(100);
        let (_shutdown_tx, shutdown_rx) = oneshot::channel();

        for _ in 0..100 {
            state.insert(rand_trits_field::<Address>(), rng.gen_range(0, 100_000_000));
        }

        spawn(LedgerStateWorker::new(state).run(rx, shutdown_rx));

        for _ in 0..100 {
            let (get_balance_tx, get_balance_rx) = oneshot::channel();
            block_on(tx.send(LedgerStateWorkerEvent::GetBalance(
                rand_trits_field::<Address>(),
                get_balance_tx,
            )))
            .unwrap();
            let value = block_on(get_balance_rx).unwrap();
            assert!(value.is_none());
        }
    }

    #[test]
    fn apply_diff_on_not_found() {
        let mut rng = rand::thread_rng();
        let mut diff = HashMap::new();
        let (mut tx, rx) = mpsc::channel(100);
        let (_shutdown_tx, shutdown_rx) = oneshot::channel();

        for _ in 0..100 {
            diff.insert(rand_trits_field::<Address>(), rng.gen_range(0, 100_000_000));
        }

        block_on(tx.send(LedgerStateWorkerEvent::ApplyDiff(diff.clone()))).unwrap();

        spawn(LedgerStateWorker::new(HashMap::new()).run(rx, shutdown_rx));

        for (address, balance) in diff {
            let (get_balance_tx, get_balance_rx) = oneshot::channel();
            block_on(tx.send(LedgerStateWorkerEvent::GetBalance(address, get_balance_tx))).unwrap();
            let value = block_on(get_balance_rx).unwrap().unwrap();
            assert_eq!(balance as u64, value)
        }
    }

    // TODO test LedgerStateWorkerEvent::ApplyDiff
}
