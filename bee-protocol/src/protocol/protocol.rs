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
    event::{LatestMilestoneChanged, LatestSolidMilestoneChanged},
    milestone::MilestoneIndex,
    peer::{Peer, PeerManager},
    protocol::ProtocolMetrics,
    tangle::tangle,
    worker::{
        BroadcasterWorker, BundleValidatorWorker, HasherWorker, KickstartWorker, MilestoneRequesterWorker,
        MilestoneResponderWorker, MilestoneSolidifierWorker, MilestoneSolidifierWorkerEvent, MilestoneValidatorWorker,
        PeerHandshakerWorker, ProcessorWorker, SolidPropagatorWorker, StatusWorker, TpsWorker,
        TransactionRequesterWorker, TransactionResponderWorker,
    },
};

use bee_common_ext::{bee_node::BeeNode, event::Bus, node::Node, worker::Worker};
use bee_crypto::ternary::Hash;
use bee_network::{EndpointId, Network, Origin};

use dashmap::DashMap;
use futures::channel::{mpsc, oneshot};
use log::{debug, info, warn};
use tokio::spawn;

use std::{net::SocketAddr, ptr, sync::Arc, time::Instant};

static mut PROTOCOL: *const Protocol = ptr::null();

pub struct Protocol {
    pub(crate) config: ProtocolConfig,
    pub(crate) network: Network,
    // TODO temporary
    pub(crate) local_snapshot_timestamp: u64,
    pub(crate) bus: Arc<Bus<'static>>,
    pub(crate) metrics: ProtocolMetrics,
    pub(crate) peer_manager: PeerManager,
    pub(crate) requested_transactions: DashMap<Hash, (MilestoneIndex, Instant)>,
    pub(crate) requested_milestones: DashMap<MilestoneIndex, Instant>,
}

impl Protocol {
    pub async fn init(
        config: ProtocolConfig,
        network: Network,
        local_snapshot_timestamp: u64,
        bee_node: &BeeNode,
        bus: Arc<Bus<'static>>,
    ) {
        if unsafe { !PROTOCOL.is_null() } {
            warn!("Already initialized.");
            return;
        }

        let protocol = Protocol {
            config,
            network: network.clone(),
            local_snapshot_timestamp,
            bus,
            metrics: ProtocolMetrics::new(),
            peer_manager: PeerManager::new(),
            requested_transactions: Default::default(),
            requested_milestones: Default::default(),
        };

        unsafe {
            PROTOCOL = Box::leak(protocol.into()) as *const _;
        }

        let (ms_send, ms_recv) = oneshot::channel();

        HasherWorker::start(bee_node, Protocol::get().config.workers.transaction_worker_cache);
        ProcessorWorker::start(bee_node, ());
        TransactionResponderWorker::start(bee_node, ());
        MilestoneResponderWorker::start(bee_node, ());
        TransactionRequesterWorker::start(bee_node, ());
        MilestoneRequesterWorker::start(bee_node, ());
        MilestoneValidatorWorker::start(bee_node, Protocol::get().config.coordinator.sponge_type);
        BroadcasterWorker::start(bee_node, network);
        BundleValidatorWorker::start(bee_node, ());
        SolidPropagatorWorker::start(bee_node, ());
        StatusWorker::start(bee_node, Protocol::get().config.workers.status_interval);
        TpsWorker::start(bee_node, ());
        KickstartWorker::start(bee_node, (ms_send, Protocol::get().config.workers.ms_sync_count));
        async { MilestoneSolidifierWorker::start(bee_node, ms_recv).await };

        Protocol::get()
            .bus
            .add_listener(|latest_milestone: &LatestMilestoneChanged| {
                info!(
                    "New milestone {} {}.",
                    *latest_milestone.0.index,
                    latest_milestone
                        .0
                        .hash()
                        .iter_trytes()
                        .map(char::from)
                        .collect::<String>()
                );
                tangle().update_latest_milestone_index(latest_milestone.0.index);

                // TODO spawn ?
                Protocol::broadcast_heartbeat(
                    tangle().get_latest_solid_milestone_index(),
                    tangle().get_pruning_index(),
                    latest_milestone.0.index,
                );
            });

        // Protocol::get()
        //     .bus
        //     .add_listener(|latest_solid_milestone: &LatestSolidMilestoneChanged| {
        // TODO block_on ?
        // TODO uncomment on Chrysalis Pt1.
        // block_on(Protocol::broadcast_heartbeat(
        //     tangle().get_latest_solid_milestone_index(),
        //     tangle().get_pruning_index(),
        // ));
        // });

        let milestone_solidifier = bee_node.worker::<MilestoneSolidifierWorker>().unwrap().tx.clone();
        let milestone_requester = bee_node.worker::<MilestoneRequesterWorker>().unwrap().tx.clone();

        Protocol::get()
            .bus
            .add_listener(move |latest_solid_milestone: &LatestSolidMilestoneChanged| {
                debug!("New solid milestone {}.", *latest_solid_milestone.0.index);
                tangle().update_latest_solid_milestone_index(latest_solid_milestone.0.index);

                let ms_sync_count = Protocol::get().config.workers.ms_sync_count;
                let next_ms = latest_solid_milestone.0.index + MilestoneIndex(ms_sync_count);

                if !tangle().is_synced() {
                    if tangle().contains_milestone(next_ms) {
                        milestone_solidifier.unbounded_send(MilestoneSolidifierWorkerEvent(next_ms));
                    } else {
                        Protocol::request_milestone(&milestone_requester, next_ms, None);
                    }
                }

                // TODO spawn ?
                Protocol::broadcast_heartbeat(
                    latest_solid_milestone.0.index,
                    tangle().get_pruning_index(),
                    tangle().get_latest_milestone_index(),
                );
            });
    }

    pub(crate) fn get() -> &'static Protocol {
        if unsafe { PROTOCOL.is_null() } {
            panic!("Uninitialized protocol.");
        } else {
            unsafe { &*PROTOCOL }
        }
    }

    pub fn register(
        bee_node: &BeeNode,
        epid: EndpointId,
        address: SocketAddr,
        origin: Origin,
    ) -> (mpsc::UnboundedSender<Vec<u8>>, oneshot::Sender<()>) {
        // TODO check if not already added ?

        let peer = Arc::new(Peer::new(epid, address, origin));

        let (receiver_tx, receiver_rx) = mpsc::unbounded();
        let (receiver_shutdown_tx, receiver_shutdown_rx) = oneshot::channel();

        Protocol::get().peer_manager.add(peer.clone());

        spawn(
            PeerHandshakerWorker::new(
                Protocol::get().network.clone(),
                peer,
                bee_node.worker::<HasherWorker>().unwrap().tx.clone(),
                bee_node.worker::<TransactionResponderWorker>().unwrap().tx.clone(),
                bee_node.worker::<MilestoneResponderWorker>().unwrap().tx.clone(),
                bee_node.worker::<MilestoneRequesterWorker>().unwrap().tx.clone(),
            )
            .run(receiver_rx, receiver_shutdown_rx),
        );

        (receiver_tx, receiver_shutdown_tx)
    }
}
