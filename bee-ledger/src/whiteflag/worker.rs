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
    event::MilestoneConfirmed,
    state::LedgerState,
    whiteflag::{
        bundle::load_bundle_builder, confirmation::Confirmation, merkle::Merkle, traversal::Error as TraversalError,
        WhiteFlag,
    },
};

use bee_common::{shutdown_stream::ShutdownStream, worker::Error as WorkerError};
use bee_crypto::ternary::Hash;
use bee_protocol::{config::ProtocolCoordinatorConfig, tangle::tangle, Milestone, MilestoneIndex};
// use bee_tangle::traversal::visit_parents_depth_first;
use bee_transaction::bundled::{Address, BundledTransactionField};

use blake2::Blake2b;
use futures::{
    channel::{mpsc, oneshot},
    stream::{Fuse, StreamExt},
};
use log::{error, info, warn};

use std::collections::HashMap;

type Receiver = ShutdownStream<Fuse<mpsc::UnboundedReceiver<LedgerWorkerEvent>>>;

enum Error {
    NonContiguousMilestone,
    InvalidConfirmationSet(TraversalError),
}

pub enum LedgerWorkerEvent {
    Confirm(Milestone),
    GetBalance(Address, oneshot::Sender<u64>),
}

pub(crate) struct LedgerWorker {
    index: MilestoneIndex,
    pub(crate) state: LedgerState,
    coo_config: ProtocolCoordinatorConfig,
    receiver: Receiver,
}

impl LedgerWorker {
    // TODO pass type
    pub fn new(
        index: MilestoneIndex,
        state: HashMap<Address, u64>,
        coo_config: ProtocolCoordinatorConfig,
        receiver: Receiver,
    ) -> Self {
        Self {
            index,
            state: LedgerState(state),
            coo_config,
            receiver,
        }
    }

    fn milestone_info(&self, hash: &Hash) -> u64 {
        // TODO handle error of both unwrap
        let ms = load_bundle_builder(hash).unwrap();
        let timestamp = ms.get(0).unwrap().get_timestamp();
        let path = ms.get(2).unwrap().payload();

        println!(
            "PROOF {:?}",
            &path
                .to_inner()
                .iter_trytes()
                .map(|trit| char::from(trit))
                .collect::<String>()
                [(self.coo_config.depth() * 81) as usize..((self.coo_config.depth() + 1) * 81) as usize]
        );

        timestamp
    }

    fn confirm(&mut self, milestone: Milestone) -> Result<(), Error> {
        if milestone.index() != MilestoneIndex(self.index.0 + 1) {
            error!("Tried to confirm {} on top of {}.", milestone.index().0, self.index.0);
            return Err(Error::NonContiguousMilestone);
        }

        info!("Confirming milestone {}.", milestone.index().0);

        let timestamp = self.milestone_info(milestone.hash());

        let mut confirmation = Confirmation::new(timestamp);

        match self.visit_bundles_dfs(*milestone.hash(), &mut confirmation) {
            Ok(_) => {
                let merkle_proof = Merkle::<Blake2b>::new().hash(&confirmation.tails_included);

                println!("proof {:?}", merkle_proof);

                self.index = milestone.index();

                // TODO debug!
                println!(
                    "Confirmed milestone {}: referenced {}, zero value {}, conflicting {}, included {}.",
                    *milestone.index(),
                    confirmation.tails_referenced.len(),
                    confirmation.num_tails_zero_value,
                    confirmation.num_tails_conflicting,
                    confirmation.tails_included.len()
                );

                // TODO this only actually confirm tails
                for hash in confirmation.tails_referenced.iter() {
                    tangle().update_metadata(&hash, |meta| {
                        meta.flags_mut().set_confirmed();
                        meta.set_milestone_index(milestone.index());
                        meta.set_confirmation_timestamp(timestamp);
                    });
                }

                // TODO would be better if we could mutate meta through traversal
                // visit_parents_depth_first(
                //     tangle(),
                //     *milestone.hash(),
                //     |_, _, meta| !meta.flags().is_confirmed(),
                //     |_, _, _| {
                //         tangle().update_metadata(milestone.hash(), |meta| {
                //             meta.flags_mut().set_confirmed();
                //             meta.set_milestone_index(milestone.index());
                //             meta.set_confirmation_timestamp(timestamp);
                //         });
                //     },
                //     |_| {},
                // );

                WhiteFlag::get().bus.dispatch(MilestoneConfirmed {
                    milestone,
                    timestamp,
                    tails_referenced: confirmation.tails_referenced.len(),
                    tails_zero_value: confirmation.num_tails_zero_value,
                    tails_conflicting: confirmation.num_tails_conflicting,
                    tails_included: confirmation.tails_included.len(),
                });

                Ok(())
            }
            Err(e) => {
                error!(
                    "Error occured while traversing to confirm {}: {:?}.",
                    milestone.index().0,
                    e
                );
                Err(Error::InvalidConfirmationSet(e))
            }
        }
    }

    fn get_balance(&self, address: Address, sender: oneshot::Sender<u64>) {
        if let Err(e) = sender.send(*self.state.get_or_zero(&address)) {
            warn!("Failed to send balance: {:?}.", e);
        }
    }

    pub async fn run(mut self) -> Result<(), WorkerError> {
        info!("Running.");

        while let Some(event) = self.receiver.next().await {
            match event {
                LedgerWorkerEvent::Confirm(milestone) => {
                    if self.confirm(milestone).is_err() {
                        panic!("Error while confirming milestone, aborting.");
                    }
                }
                LedgerWorkerEvent::GetBalance(address, sender) => self.get_balance(address, sender),
            }
        }

        info!("Stopped.");

        Ok(())
    }
}

// #[cfg(test)]
// mod tests {
//
//     use super::*;
//
//     use bee_test::field::rand_trits_field;
//
//     use async_std::task::{block_on, spawn};
//     use futures::sink::SinkExt;
//     use rand::Rng;
//
//     #[test]
//     fn get_balances() {
//         let mut rng = rand::thread_rng();
//         let mut state = HashMap::new();
//         let (mut tx, rx) = mpsc::unbounded();
//         let (_shutdown_tx, shutdown_rx) = oneshot::channel();
//
//         for _ in 0..100 {
//             state.insert(rand_trits_field::<Address>(), rng.gen_range(0, 100_000_000));
//         }
//
//         spawn(LedgerStateWorker::new(state.clone(), ShutdownStream::new(shutdown_rx, rx)).run());
//
//         for (address, balance) in state {
//             let (get_balance_tx, get_balance_rx) = oneshot::channel();
//             block_on(tx.send(LedgerStateWorkerEvent::GetBalance(address, get_balance_tx))).unwrap();
//             let value = block_on(get_balance_rx).unwrap().unwrap();
//             assert_eq!(balance, value)
//         }
//     }
//
//     #[test]
//     fn get_balances_not_found() {
//         let mut rng = rand::thread_rng();
//         let mut state = HashMap::new();
//         let (mut tx, rx) = mpsc::unbounded();
//         let (_shutdown_tx, shutdown_rx) = oneshot::channel();
//
//         for _ in 0..100 {
//             state.insert(rand_trits_field::<Address>(), rng.gen_range(0, 100_000_000));
//         }
//
//         spawn(LedgerStateWorker::new(state.clone(), ShutdownStream::new(shutdown_rx, rx)).run());
//
//         for _ in 0..100 {
//             let (get_balance_tx, get_balance_rx) = oneshot::channel();
//             block_on(tx.send(LedgerStateWorkerEvent::GetBalance(
//                 rand_trits_field::<Address>(),
//                 get_balance_tx,
//             )))
//             .unwrap();
//             let value = block_on(get_balance_rx).unwrap();
//             assert!(value.is_none());
//         }
//     }
//
//     #[test]
//     fn apply_diff_on_not_found() {
//         let mut rng = rand::thread_rng();
//         let mut diff = HashMap::new();
//         let (mut tx, rx) = mpsc::unbounded();
//         let (_shutdown_tx, shutdown_rx) = oneshot::channel();
//
//         for _ in 0..100 {
//             diff.insert(rand_trits_field::<Address>(), rng.gen_range(0, 100_000_000));
//         }
//
//         block_on(tx.send(LedgerStateWorkerEvent::ApplyDiff(diff.clone()))).unwrap();
//
//         spawn(LedgerStateWorker::new(HashMap::new(), ShutdownStream::new(shutdown_rx, rx)).run());
//
//         for (address, balance) in diff {
//             let (get_balance_tx, get_balance_rx) = oneshot::channel();
//             block_on(tx.send(LedgerStateWorkerEvent::GetBalance(address, get_balance_tx))).unwrap();
//             let value = block_on(get_balance_rx).unwrap().unwrap();
//             assert_eq!(balance as u64, value)
//         }
//     }
//
//     // TODO test LedgerStateWorkerEvent::ApplyDiff
// }
