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
    merkle_hasher::MerkleHasher,
    metadata::WhiteFlagMetadata,
    white_flag::{visit_dfs, Error as TraversalError},
};

use bee_common::{shutdown_stream::ShutdownStream, worker::Error as WorkerError};
use bee_common_ext::{
    event::Bus,
    node::{Node, ResHandle},
    worker::Worker,
};
use bee_message::{payload::Payload, MessageId};
use bee_protocol::{config::ProtocolCoordinatorConfig, tangle::MsTangle, MilestoneIndex, StorageWorker, TangleWorker};
use bee_storage::storage::Backend;

use async_trait::async_trait;
use blake2::Blake2b;
use futures::stream::StreamExt;
use log::{error, info};

use std::{any::TypeId, sync::Arc};

const MERKLE_PROOF_LENGTH: usize = 384;

enum Error {
    MilestoneMessageNotFound,
    NoMilestonePayload,
    NonContiguousMilestone,
    InvalidConfirmationSet(TraversalError),
    MerkleProofMismatch,
    InvalidMessagesCount,
}

pub enum LedgerWorkerEvent {
    Confirm(MessageId),
    // GetBalance(Address, oneshot::Sender<u64>),
}

pub(crate) struct LedgerWorker {
    pub(crate) tx: flume::Sender<LedgerWorkerEvent>,
}

async fn confirm<B: Backend>(
    tangle: &MsTangle<B>,
    storage: &ResHandle<B>,
    message_id: MessageId,
    index: &mut MilestoneIndex,
    coo_config: &ProtocolCoordinatorConfig,
    bus: &Arc<Bus<'static>>,
) -> Result<(), Error> {
    let message = tangle.get(&message_id).await.ok_or(Error::MilestoneMessageNotFound)?;

    let milestone = match message.payload() {
        Payload::Milestone(milestone) => milestone,
        _ => return Err(Error::NoMilestonePayload),
    };

    if milestone.essence().index() != index.0 + 1 {
        error!(
            "Tried to confirm {} on top of {}.",
            milestone.essence().index(),
            index.0
        );
        return Err(Error::NonContiguousMilestone);
    }

    let mut metadata = WhiteFlagMetadata::new(
        MilestoneIndex(milestone.essence().index()),
        milestone.essence().timestamp(),
    );

    if let Err(e) = visit_dfs(tangle, storage, message_id, &mut metadata).await {
        error!(
            "Error occured while traversing to confirm {}: {:?}.",
            milestone.essence().index(),
            e
        );
        return Err(Error::InvalidConfirmationSet(e));
    };

    let merkle_proof = MerkleHasher::<Blake2b>::new().digest(&metadata.messages_included);

    if !merkle_proof.eq(&milestone.essence().merkle_proof()) {
        error!(
            "The computed merkle proof on milestone {}, {}, does not match the one provided by the coordinator, {}.",
            hex::encode(merkle_proof),
            milestone.essence().index(),
            hex::encode(milestone.essence().merkle_proof())
        );
        return Err(Error::MerkleProofMismatch);
    }

    if metadata.num_messages_referenced
        != metadata.num_messages_excluded_no_transaction
            + metadata.num_messages_excluded_conflicting
            + metadata.messages_included.len()
    {
        error!(
            "Invalid messages count at {}: referenced ({}) != no transaction ({}) + conflicting ({}) + included ({}).",
            milestone.essence().index(),
            metadata.num_messages_referenced,
            metadata.num_messages_excluded_no_transaction,
            metadata.num_messages_excluded_conflicting,
            metadata.messages_included.len()
        );
        return Err(Error::InvalidMessagesCount);
    }

    *index = MilestoneIndex(milestone.essence().index());

    info!(
        "Confirmed milestone {}: referenced {}, zero value {}, conflicting {}, included {}.",
        milestone.essence().index(),
        metadata.num_messages_referenced,
        metadata.num_messages_excluded_no_transaction,
        metadata.num_messages_excluded_conflicting,
        metadata.messages_included.len()
    );

    bus.dispatch(MilestoneConfirmed {
        milestone,
        timestamp: milestone.essence().timestamp(),
        messages_referenced: metadata.num_messages_referenced,
        messages_excluded_no_transaction: metadata.num_messages_excluded_no_transaction,
        messages_excluded_conflicting: metadata.num_messages_excluded_conflicting,
        messages_included: metadata.messages_included.len(),
    });

    Ok(())
}

// fn get_balance(state: &LedgerState, address: Address, sender: oneshot::Sender<u64>) {
//     if let Err(e) = sender.send(state.get_or_zero(&address)) {
//         warn!("Failed to send balance: {:?}.", e);
//     }
// }

#[async_trait]
impl<N: Node> Worker<N> for LedgerWorker {
    type Config = (MilestoneIndex, ProtocolCoordinatorConfig, Arc<Bus<'static>>);
    type Error = WorkerError;

    fn dependencies() -> &'static [TypeId] {
        vec![TypeId::of::<TangleWorker>(), TypeId::of::<StorageWorker>()].leak()
    }

    async fn start(node: &mut N, config: Self::Config) -> Result<Self, Self::Error> {
        let (tx, rx) = flume::unbounded();

        let tangle = node.resource::<MsTangle<N::Backend>>();
        let storage = node.storage();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, rx.into_stream());

            let mut index = config.0;
            let coo_config = config.1;
            let bus = config.2;

            while let Some(event) = receiver.next().await {
                match event {
                    LedgerWorkerEvent::Confirm(message_id) => {
                        if confirm(&tangle, storage, message_id, &mut index, &coo_config, &bus)
                            .await
                            .is_err()
                        {
                            panic!(
                                "Error while confirming milestone from message {}, aborting.",
                                message_id
                            );
                        }
                    } // LedgerWorkerEvent::GetBalance(address, sender) => get_balance(&state, address, sender),
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
//         let (mut tx, rx) = flume::unbounded();
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
//         let (mut tx, rx) = flume::unbounded();
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
//         let (mut tx, rx) = flume::unbounded();
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
