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
    config::ProtocolConfig,
    milestone::MilestoneIndex,
    peer::{Peer, PeerManager},
    protocol::ProtocolMetrics,
    util::WaitPriorityQueue,
    worker::{
        BroadcasterWorker, BroadcasterWorkerEvent, MilestoneRequesterWorker, MilestoneRequesterWorkerEntry,
        MilestoneResponderWorker, MilestoneResponderWorkerEvent, MilestoneSolidifierWorker,
        MilestoneSolidifierWorkerEvent, MilestoneValidatorWorker, MilestoneValidatorWorkerEvent, PeerHandshakerWorker,
        StatusWorker, TpsWorker, TransactionRequesterWorker, TransactionRequesterWorkerEntry,
        TransactionResponderWorker, TransactionResponderWorkerEvent, TransactionSolidifierWorker,
        TransactionSolidifierWorkerEvent, TransactionWorker, TransactionWorkerEvent,
    },
};

use bee_crypto::{CurlP27, CurlP81, Kerl, SpongeType};
use bee_network::{Address, EndpointId, Network, Origin};
use bee_signing::WotsPublicKey;
use bee_transaction::Hash;

use std::{
    ptr,
    sync::{Arc, Mutex},
};

use async_std::task::spawn;
use dashmap::DashMap;
use futures::channel::{mpsc, oneshot};
use log::warn;

static mut PROTOCOL: *const Protocol = ptr::null();

pub struct Protocol {
    pub(crate) config: ProtocolConfig,
    pub(crate) network: Network,
    pub(crate) metrics: ProtocolMetrics,
    pub(crate) transaction_worker: (mpsc::Sender<TransactionWorkerEvent>, Mutex<Option<oneshot::Sender<()>>>),
    pub(crate) transaction_responder_worker: (
        mpsc::Sender<TransactionResponderWorkerEvent>,
        Mutex<Option<oneshot::Sender<()>>>,
    ),
    pub(crate) milestone_responder_worker: (
        mpsc::Sender<MilestoneResponderWorkerEvent>,
        Mutex<Option<oneshot::Sender<()>>>,
    ),
    pub(crate) transaction_requester_worker: (
        WaitPriorityQueue<TransactionRequesterWorkerEntry>,
        Mutex<Option<oneshot::Sender<()>>>,
    ),
    pub(crate) milestone_requester_worker: (
        WaitPriorityQueue<MilestoneRequesterWorkerEntry>,
        Mutex<Option<oneshot::Sender<()>>>,
    ),
    pub(crate) milestone_validator_worker: (
        mpsc::Sender<MilestoneValidatorWorkerEvent>,
        Mutex<Option<oneshot::Sender<()>>>,
    ),
    pub(crate) transaction_solidifier_worker: (
        mpsc::Sender<TransactionSolidifierWorkerEvent>,
        Mutex<Option<oneshot::Sender<()>>>,
    ),
    pub(crate) milestone_solidifier_worker: (
        mpsc::Sender<MilestoneSolidifierWorkerEvent>,
        Mutex<Option<oneshot::Sender<()>>>,
    ),
    pub(crate) broadcaster_worker: (mpsc::Sender<BroadcasterWorkerEvent>, Mutex<Option<oneshot::Sender<()>>>),
    pub(crate) status_worker: Mutex<Option<oneshot::Sender<()>>>,
    pub(crate) tps_worker: Mutex<Option<oneshot::Sender<()>>>,
    pub(crate) peer_manager: PeerManager,
    pub(crate) requested: DashMap<Hash, MilestoneIndex>,
}

impl Protocol {
    pub async fn init(config: ProtocolConfig, network: Network) {
        if unsafe { !PROTOCOL.is_null() } {
            warn!("Already initialized.");
            return;
        }

        crate::tangle::init();

        let (transaction_worker_tx, transaction_worker_rx) = mpsc::channel(config.workers.transaction_worker_bound);
        let (transaction_worker_shutdown_tx, transaction_worker_shutdown_rx) = oneshot::channel();

        let (transaction_responder_worker_tx, transaction_responder_worker_rx) =
            mpsc::channel(config.workers.transaction_responder_worker_bound);
        let (transaction_responder_worker_shutdown_tx, transaction_responder_worker_shutdown_rx) = oneshot::channel();

        let (milestone_responder_worker_tx, milestone_responder_worker_rx) =
            mpsc::channel(config.workers.milestone_responder_worker_bound);
        let (milestone_responder_worker_shutdown_tx, milestone_responder_worker_shutdown_rx) = oneshot::channel();

        let (transaction_requester_worker_shutdown_tx, transaction_requester_worker_shutdown_rx) = oneshot::channel();

        let (milestone_requester_worker_shutdown_tx, milestone_requester_worker_shutdown_rx) = oneshot::channel();

        let (milestone_validator_worker_tx, milestone_validator_worker_rx) =
            mpsc::channel(config.workers.milestone_validator_worker_bound);
        let (milestone_validator_worker_shutdown_tx, milestone_validator_worker_shutdown_rx) = oneshot::channel();

        let (transaction_solidifier_worker_tx, transaction_solidifier_worker_rx) =
            mpsc::channel(config.workers.transaction_solidifier_worker_bound);
        let (transaction_solidifier_worker_shutdown_tx, transaction_solidifier_worker_shutdown_rx) = oneshot::channel();

        let (milestone_solidifier_worker_tx, milestone_solidifier_worker_rx) =
            mpsc::channel(config.workers.milestone_solidifier_worker_bound);
        let (milestone_solidifier_worker_shutdown_tx, milestone_solidifier_worker_shutdown_rx) = oneshot::channel();

        let (broadcaster_worker_tx, broadcaster_worker_rx) = mpsc::channel(config.workers.broadcaster_worker_bound);
        let (broadcaster_worker_shutdown_tx, broadcaster_worker_shutdown_rx) = oneshot::channel();

        let (status_worker_shutdown_tx, status_worker_shutdown_rx) = oneshot::channel();

        let (tps_worker_shutdown_tx, tps_worker_shutdown_rx) = oneshot::channel();

        let protocol = Protocol {
            config,
            network: network.clone(),
            metrics: ProtocolMetrics::new(),
            transaction_worker: (transaction_worker_tx, Mutex::new(Some(transaction_worker_shutdown_tx))),
            transaction_responder_worker: (
                transaction_responder_worker_tx,
                Mutex::new(Some(transaction_responder_worker_shutdown_tx)),
            ),
            milestone_responder_worker: (
                milestone_responder_worker_tx,
                Mutex::new(Some(milestone_responder_worker_shutdown_tx)),
            ),
            transaction_requester_worker: (
                Default::default(),
                Mutex::new(Some(transaction_requester_worker_shutdown_tx)),
            ),
            milestone_requester_worker: (
                Default::default(),
                Mutex::new(Some(milestone_requester_worker_shutdown_tx)),
            ),
            milestone_validator_worker: (
                milestone_validator_worker_tx,
                Mutex::new(Some(milestone_validator_worker_shutdown_tx)),
            ),
            transaction_solidifier_worker: (
                transaction_solidifier_worker_tx,
                Mutex::new(Some(transaction_solidifier_worker_shutdown_tx)),
            ),
            milestone_solidifier_worker: (
                milestone_solidifier_worker_tx,
                Mutex::new(Some(milestone_solidifier_worker_shutdown_tx)),
            ),
            broadcaster_worker: (broadcaster_worker_tx, Mutex::new(Some(broadcaster_worker_shutdown_tx))),
            status_worker: Mutex::new(Some(status_worker_shutdown_tx)),
            tps_worker: Mutex::new(Some(tps_worker_shutdown_tx)),
            peer_manager: PeerManager::new(network.clone()),
            requested: Default::default(),
        };

        unsafe {
            PROTOCOL = Box::leak(protocol.into()) as *const _;
        }

        spawn(
            TransactionWorker::new(
                Protocol::get().milestone_validator_worker.0.clone(),
                Protocol::get().config.workers.transaction_worker_cache,
            )
            .run(transaction_worker_rx, transaction_worker_shutdown_rx),
        );
        spawn(TransactionResponderWorker::new().run(
            transaction_responder_worker_rx,
            transaction_responder_worker_shutdown_rx,
        ));
        spawn(
            MilestoneResponderWorker::new().run(milestone_responder_worker_rx, milestone_responder_worker_shutdown_rx),
        );
        spawn(TransactionRequesterWorker::new().run(transaction_requester_worker_shutdown_rx));
        spawn(MilestoneRequesterWorker::new().run(milestone_requester_worker_shutdown_rx));

        match Protocol::get().config.coordinator.sponge_type {
            SpongeType::Kerl => spawn(
                MilestoneValidatorWorker::<Kerl, WotsPublicKey<Kerl>>::new()
                    .run(milestone_validator_worker_rx, milestone_validator_worker_shutdown_rx),
            ),
            SpongeType::CurlP27 => spawn(
                MilestoneValidatorWorker::<CurlP27, WotsPublicKey<CurlP27>>::new()
                    .run(milestone_validator_worker_rx, milestone_validator_worker_shutdown_rx),
            ),
            SpongeType::CurlP81 => spawn(
                MilestoneValidatorWorker::<CurlP81, WotsPublicKey<CurlP81>>::new()
                    .run(milestone_validator_worker_rx, milestone_validator_worker_shutdown_rx),
            ),
        };

        spawn(TransactionSolidifierWorker::new().run(
            transaction_solidifier_worker_rx,
            transaction_solidifier_worker_shutdown_rx,
        ));
        spawn(
            MilestoneSolidifierWorker::new()
                .run(milestone_solidifier_worker_rx, milestone_solidifier_worker_shutdown_rx),
        );
        spawn(BroadcasterWorker::new(network).run(broadcaster_worker_rx, broadcaster_worker_shutdown_rx));
        spawn(StatusWorker::new(Protocol::get().config.workers.status_interval).run(status_worker_shutdown_rx));
        spawn(TpsWorker::new().run(tps_worker_shutdown_rx));
    }

    pub async fn shutdown() {
        if let Ok(mut shutdown) = Protocol::get().transaction_worker.1.lock() {
            if let Some(shutdown) = shutdown.take() {
                if let Err(e) = shutdown.send(()) {
                    warn!("Shutting down TransactionWorker failed: {:?}.", e);
                }
            }
        }
        if let Ok(mut shutdown) = Protocol::get().transaction_responder_worker.1.lock() {
            if let Some(shutdown) = shutdown.take() {
                if let Err(e) = shutdown.send(()) {
                    warn!("Shutting down TransactionResponderWorker failed: {:?}.", e);
                }
            }
        }
        if let Ok(mut shutdown) = Protocol::get().milestone_responder_worker.1.lock() {
            if let Some(shutdown) = shutdown.take() {
                if let Err(e) = shutdown.send(()) {
                    warn!("Shutting down MilestoneResponderWorker failed: {:?}.", e);
                }
            }
        }
        if let Ok(mut shutdown) = Protocol::get().transaction_requester_worker.1.lock() {
            if let Some(shutdown) = shutdown.take() {
                if let Err(e) = shutdown.send(()) {
                    warn!("Shutting down TransactionRequesterWorker failed: {:?}.", e);
                }
            }
        }
        if let Ok(mut shutdown) = Protocol::get().milestone_requester_worker.1.lock() {
            if let Some(shutdown) = shutdown.take() {
                if let Err(e) = shutdown.send(()) {
                    warn!("Shutting down MilestoneRequesterWorker failed: {:?}.", e);
                }
            }
        }
        if let Ok(mut shutdown) = Protocol::get().milestone_validator_worker.1.lock() {
            if let Some(shutdown) = shutdown.take() {
                if let Err(e) = shutdown.send(()) {
                    warn!("Shutting down MilestoneValidatorWorker failed: {:?}.", e);
                }
            }
        }
        if let Ok(mut shutdown) = Protocol::get().transaction_solidifier_worker.1.lock() {
            if let Some(shutdown) = shutdown.take() {
                if let Err(e) = shutdown.send(()) {
                    warn!("Shutting down TransactionSolidifierWorker failed: {:?}.", e);
                }
            }
        }
        if let Ok(mut shutdown) = Protocol::get().milestone_solidifier_worker.1.lock() {
            if let Some(shutdown) = shutdown.take() {
                if let Err(e) = shutdown.send(()) {
                    warn!("Shutting down MilestoneSolidifierWorker failed: {:?}.", e);
                }
            }
        }
        if let Ok(mut shutdown) = Protocol::get().broadcaster_worker.1.lock() {
            if let Some(shutdown) = shutdown.take() {
                if let Err(e) = shutdown.send(()) {
                    warn!("Shutting down BroadcasterWorker failed: {:?}.", e);
                }
            }
        }
        if let Ok(mut shutdown) = Protocol::get().status_worker.lock() {
            if let Some(shutdown) = shutdown.take() {
                if let Err(e) = shutdown.send(()) {
                    warn!("Shutting down StatusWorker failed: {:?}.", e);
                }
            }
        }
        if let Ok(mut shutdown) = Protocol::get().tps_worker.lock() {
            if let Some(shutdown) = shutdown.take() {
                if let Err(e) = shutdown.send(()) {
                    warn!("Shutting down TpsWorker failed: {:?}.", e);
                }
            }
        }
    }

    pub(crate) fn get() -> &'static Protocol {
        if unsafe { PROTOCOL.is_null() } {
            panic!("Uninitialized protocol.");
        } else {
            unsafe { &*PROTOCOL }
        }
    }

    pub fn register(
        epid: EndpointId,
        address: Address,
        origin: Origin,
    ) -> (mpsc::Sender<Vec<u8>>, oneshot::Sender<()>) {
        // TODO check if not already added ?

        let peer = Arc::new(Peer::new(epid, address, origin));

        let (receiver_tx, receiver_rx) = mpsc::channel(Protocol::get().config.workers.receiver_worker_bound);
        let (receiver_shutdown_tx, receiver_shutdown_rx) = oneshot::channel();

        Protocol::get().peer_manager.add(peer.clone());

        spawn(PeerHandshakerWorker::new(Protocol::get().network.clone(), peer).run(receiver_rx, receiver_shutdown_rx));

        (receiver_tx, receiver_shutdown_tx)
    }
}
