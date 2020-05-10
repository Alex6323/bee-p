use bee_bundle::Address;

use std::collections::HashMap;

use futures::{
    channel::{mpsc, oneshot},
    future::FutureExt,
    select,
    stream::StreamExt,
};
use log::{info, warn};

pub enum LedgerWorkerEvent {
    ApplyDiff(HashMap<Address, i64>),
    GetBalance(Address, oneshot::Sender<Option<u64>>),
}

pub struct LedgerWorker {
    state: HashMap<Address, u64>,
}

impl LedgerWorker {
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
                        warn!("[LedgerWorker ] Ignoring conflicting diff.");
                    }
                })
                .or_default();
        }
    }

    fn get_balance(&self, address: Address, sender: oneshot::Sender<Option<u64>>) {
        if let Err(e) = sender.send(self.state.get(&address).cloned()) {
            warn!("[LedgerWorker ] Failed to send balance: {:?}.", e);
        }
    }

    pub async fn run(mut self, receiver: mpsc::Receiver<LedgerWorkerEvent>, shutdown: oneshot::Receiver<()>) {
        info!("[LedgerWorker ] Running.");

        let mut receiver_fused = receiver.fuse();
        let mut shutdown_fused = shutdown.fuse();

        loop {
            select! {
                event = receiver_fused.next() => {
                    if let Some(event) = event {
                        match event {
                            LedgerWorkerEvent::ApplyDiff(diff) => self.apply_diff(diff),
                            LedgerWorkerEvent::GetBalance(address, sender) => self.get_balance(address, sender)
                        }
                    }
                },
                _ = shutdown_fused => {
                    break;
                }
            }
        }

        info!("[LedgerWorker ] Stopped.");
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

        spawn(LedgerWorker::new(state.clone()).run(rx, shutdown_rx));

        for (address, balance) in state {
            let (get_balance_tx, get_balance_rx) = oneshot::channel();
            block_on(tx.send(LedgerWorkerEvent::GetBalance(address, get_balance_tx))).unwrap();
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

        spawn(LedgerWorker::new(state.clone()).run(rx, shutdown_rx));

        for _ in 0..100 {
            let (get_balance_tx, get_balance_rx) = oneshot::channel();
            block_on(tx.send(LedgerWorkerEvent::GetBalance(
                rand_trits_field::<Address>(),
                get_balance_tx,
            )))
            .unwrap();
            let value = block_on(get_balance_rx).unwrap();
            assert!(value.is_none());
        }
    }

    // TODO test LedgerWorkerEvent::ApplyDiff
}
