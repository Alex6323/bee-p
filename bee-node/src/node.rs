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
    config::NodeConfig,
    constants::{BEE_GIT_COMMIT, BEE_NAME, BEE_VERSION},
};

use bee_bundle::Hash;
use bee_common::logger;
use bee_ledger::{LedgerWorker, LedgerWorkerEvent};
use bee_network::{Address, Command::Connect, EndpointId, Event, EventSubscriber, Network, Origin, Shutdown};
use bee_peering::{PeerManager, StaticPeerManager};
use bee_protocol::Protocol;
use bee_snapshot::{SnapshotMetadata, SnapshotState};
use bee_tangle::tangle;

use std::collections::HashMap;

use async_std::task::{block_on, spawn};
use chrono::{offset::TimeZone, Utc};
use futures::{
    channel::{mpsc, oneshot},
    sink::SinkExt,
    stream::StreamExt,
};
use log::*;

pub struct Node {
    config: NodeConfig,
    network: Network,
    shutdown: Shutdown,
    events: EventSubscriber,
    ledger: Option<(mpsc::Sender<LedgerWorkerEvent>, oneshot::Sender<()>)>,
    peers: HashMap<EndpointId, (mpsc::Sender<Vec<u8>>, oneshot::Sender<()>)>,
}

impl Node {
    pub fn new(config: NodeConfig, network: Network, shutdown: Shutdown, events: EventSubscriber) -> Self {
        Self {
            config,
            network,
            shutdown,
            events,
            ledger: None,
            peers: HashMap::new(),
        }
    }

    async fn endpoint_added_handler(&mut self, epid: EndpointId) {
        info!("[Node ] Endpoint {} has been added.", epid);

        if let Err(e) = self.network.send(Connect { epid, responder: None }).await {
            warn!("[Node ] Sending Command::Connect for {} failed: {}.", epid, e);
        }
    }

    async fn endpoint_removed_handler(&mut self, epid: EndpointId) {
        info!("[Node ] Endpoint {} has been removed.", epid);
    }

    async fn endpoint_connected_handler(&mut self, epid: EndpointId, address: Address, origin: Origin) {
        let (receiver_tx, receiver_shutdown_tx) = Protocol::register(epid, address, origin);

        self.peers.insert(epid, (receiver_tx, receiver_shutdown_tx));
    }

    async fn endpoint_disconnected_handler(&mut self, epid: EndpointId) {
        // TODO unregister ?
        if let Some((_, shutdown)) = self.peers.remove(&epid) {
            if let Err(e) = shutdown.send(()) {
                warn!("[Node ] Sending shutdown to {} failed: {:?}.", epid, e);
            }
        }
    }

    async fn endpoint_bytes_received_handler(&mut self, epid: EndpointId, bytes: Vec<u8>) {
        if let Some(peer) = self.peers.get_mut(&epid) {
            if let Err(e) = peer.0.send(bytes).await {
                warn!("[Node ] Sending PeerWorkerEvent::Message to {} failed: {}.", epid, e);
            }
        }
    }

    pub async fn run(mut self) {
        info!("[Node ] Running.");

        while let Some(event) = self.events.next().await {
            debug!("[Node ] Received event {}.", event);

            match event {
                Event::EndpointAdded { epid, .. } => self.endpoint_added_handler(epid).await,
                Event::EndpointRemoved { epid, .. } => self.endpoint_removed_handler(epid).await,
                Event::EndpointConnected {
                    epid, origin, address, ..
                } => self.endpoint_connected_handler(epid, address, origin).await,
                Event::EndpointDisconnected { epid, .. } => self.endpoint_disconnected_handler(epid).await,
                Event::MessageReceived { epid, bytes, .. } => self.endpoint_bytes_received_handler(epid, bytes).await,
                _ => warn!("[Node ] Unsupported event {}.", event),
            }
        }
    }

    pub async fn init(&mut self) {
        logger::init(self.config.log_level);

        info!("[Node ] {} v{}-{}.", BEE_NAME, BEE_VERSION, &BEE_GIT_COMMIT[0..7]);
        info!("[Node ] Initializing...");

        block_on(StaticPeerManager::new(self.config.peering.r#static.clone(), self.network.clone()).run());

        bee_tangle::init();

        info!("[Node ] Reading snapshot metadata...");
        match SnapshotMetadata::new(self.config.snapshot.meta_file_path()) {
            Ok(snapshot_metadata) => {
                info!(
                    "[Node ] Read snapshot metadata from {} with index {}, {} solid entry points and {} seen milestones.",
                    Utc.timestamp(snapshot_metadata.timestamp() as i64, 0).to_rfc2822(),
                    snapshot_metadata.index(),
                    snapshot_metadata.solid_entry_points().len(),
                    snapshot_metadata.seen_milestones().len(),
                );
                tangle().update_solid_milestone_index(snapshot_metadata.index().into());
                // TODO get from database
                tangle().update_snapshot_milestone_index(snapshot_metadata.index().into());
                tangle().add_solid_entry_point(Hash::zeros());
                for solid_entry_point in snapshot_metadata.solid_entry_points() {
                    tangle().add_solid_entry_point(*solid_entry_point);
                }
                for seen_milestone in snapshot_metadata.seen_milestones() {
                    // TODO request ?
                }
            }
            // TODO exit ?
            Err(e) => error!(
                "[Node ] Failed to read snapshot metadata file \"{}\": {:?}.",
                self.config.snapshot.meta_file_path(),
                e
            ),
        }

        info!("[Node ] Reading snapshot state...");
        let snapshot_state = match SnapshotState::new(self.config.snapshot.state_file_path()) {
            Ok(snapshot_state) => {
                info!(
                    "[Node ] Read snapshot state with {} entries and correct supply.",
                    snapshot_state.state().len()
                );
                snapshot_state
            }
            // TODO exit ?
            Err(e) => {
                error!(
                    "[Node ] Failed to read snapshot state file \"{}\": {:?}.",
                    self.config.snapshot.state_file_path(),
                    e
                );
                panic!("TODO")
            }
        };

        Protocol::init(self.config.protocol.clone(), self.network.clone()).await;

        // TODO config
        let (ledger_worker_tx, ledger_worker_rx) = mpsc::channel(1000);
        let (ledger_worker_shutdown_tx, ledger_worker_shutdown_rx) = oneshot::channel();
        self.ledger.replace((ledger_worker_tx, ledger_worker_shutdown_tx));
        spawn(LedgerWorker::new(snapshot_state.into_state()).run(ledger_worker_rx, ledger_worker_shutdown_rx));

        info!("[Node ] Initialized.");
    }
}

#[cfg(test)]
mod tests {}
