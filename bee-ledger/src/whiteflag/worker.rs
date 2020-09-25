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
        b1t6::decode,
        merkle_hasher::MerkleHasher,
        metadata::WhiteFlagMetadata,
        traversal::{visit_bundles_dfs, Error as TraversalError},
    },
};

use bee_common::{shutdown_stream::ShutdownStream, worker::Error as WorkerError};
use bee_common_ext::{event::Bus, node::Node, worker::Worker};
use bee_crypto::ternary::{Hash, HASH_LENGTH};
use bee_protocol::{config::ProtocolCoordinatorConfig, tangle::tangle, Milestone, MilestoneIndex};
use bee_tangle::helper::load_bundle_builder;
use bee_transaction::bundled::{Address, BundledTransactionField};

use async_trait::async_trait;
use blake2::Blake2b;
use futures::{
    channel::{mpsc, oneshot},
    stream::StreamExt,
};
use log::{error, info, warn};

use std::sync::Arc;

const MERKLE_PROOF_LENGTH: usize = 384;

enum Error {
    NonContiguousMilestone,
    MerkleProofMismatch,
    InvalidTailsCount,
    InvalidConfirmationSet(TraversalError),
}

pub enum LedgerWorkerEvent {
    Confirm(Milestone),
    GetBalance(Address, oneshot::Sender<u64>),
}

pub(crate) struct LedgerWorker {
    pub(crate) tx: mpsc::UnboundedSender<LedgerWorkerEvent>,
}

fn milestone_info(hash: &Hash, coo_config: &ProtocolCoordinatorConfig) -> (Vec<u8>, u64) {
    // TODO handle error of both unwrap
    let ms = load_bundle_builder(tangle(), hash).unwrap();
    let timestamp = ms.get(0).unwrap().get_timestamp();
    let proof = decode(ms.get(2).unwrap().payload().to_inner().subslice(
        (coo_config.depth() as usize * HASH_LENGTH)..(coo_config.depth() as usize * HASH_LENGTH + MERKLE_PROOF_LENGTH),
    ));

    (proof, timestamp)
}

fn confirm(
    milestone: Milestone,
    index: &mut MilestoneIndex,
    state: &mut LedgerState,
    coo_config: &ProtocolCoordinatorConfig,
    bus: &Arc<Bus<'static>>,
) -> Result<(), Error> {
    if milestone.index() != MilestoneIndex(index.0 + 1) {
        error!("Tried to confirm {} on top of {}.", milestone.index().0, index.0);
        return Err(Error::NonContiguousMilestone);
    }

    let (merkle_proof, timestamp) = milestone_info(milestone.hash(), coo_config);

    let mut confirmation = WhiteFlagMetadata::new(milestone.index(), timestamp);

    match visit_bundles_dfs(state, *milestone.hash(), &mut confirmation) {
        Ok(_) => {
            if !merkle_proof.eq(&MerkleHasher::<Blake2b>::new().digest(&confirmation.tails_included)) {
                error!(
                    "The computed merkle proof on milestone {} does not match the one provided by the coordinator.",
                    milestone.index().0,
                );
                return Err(Error::MerkleProofMismatch);
            }

            if confirmation.num_tails_referenced
                != confirmation.num_tails_zero_value
                    + confirmation.num_tails_conflicting
                    + confirmation.tails_included.len()
            {
                error!(
                    "Invalid tails count at {}: referenced ({}) != zero ({}) + conflicting ({}) + included ({}).",
                    milestone.index().0,
                    confirmation.num_tails_referenced,
                    confirmation.num_tails_zero_value,
                    confirmation.num_tails_conflicting,
                    confirmation.tails_included.len()
                );
                return Err(Error::InvalidTailsCount);
            }

            *index = milestone.index();

            info!(
                "Confirmed milestone {}: referenced {}, zero value {}, conflicting {}, included {}.",
                *milestone.index(),
                confirmation.num_tails_referenced,
                confirmation.num_tails_zero_value,
                confirmation.num_tails_conflicting,
                confirmation.tails_included.len()
            );

            bus.dispatch(MilestoneConfirmed {
                milestone,
                timestamp,
                tails_referenced: confirmation.num_tails_referenced,
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

fn get_balance(state: &LedgerState, address: Address, sender: oneshot::Sender<u64>) {
    if let Err(e) = sender.send(state.get_or_zero(&address)) {
        warn!("Failed to send balance: {:?}.", e);
    }
}

#[async_trait]
impl<N: Node> Worker<N> for LedgerWorker {
    type Config = (
        MilestoneIndex,
        LedgerState,
        ProtocolCoordinatorConfig,
        Arc<Bus<'static>>,
    );
    type Error = WorkerError;

    async fn start(node: &N, config: Self::Config) -> Result<Self, Self::Error> {
        let (tx, rx) = mpsc::unbounded();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, rx);

            let mut index = config.0;
            let mut state = config.1;
            let coo_config = config.2;
            let bus = config.3;

            while let Some(event) = receiver.next().await {
                match event {
                    LedgerWorkerEvent::Confirm(milestone) => {
                        if confirm(milestone, &mut index, &mut state, &coo_config, &bus).is_err() {
                            panic!("Error while confirming milestone, aborting.");
                        }
                    }
                    LedgerWorkerEvent::GetBalance(address, sender) => get_balance(&state, address, sender),
                }
            }

            info!("Stopped.");
        });

        Ok(Self { tx })
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
