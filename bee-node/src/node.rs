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
    constants::{BEE_GIT_COMMIT, BEE_VERSION},
};

use bee_ledger::{LedgerWorker, LedgerWorkerEvent};
use bee_network::{self, Address, Command::Connect, EndpointId, Event, EventSubscriber, Network, Origin, Shutdown};
use bee_peering::{PeerManager, StaticPeerManager};
use bee_protocol::Protocol;
use bee_snapshot::LocalSnapshot;
use bee_tangle::tangle;
use bee_transaction::Hash;

use async_std::task::{block_on, spawn};
use chrono::{offset::TimeZone, Utc};
use futures::{
    channel::{mpsc, oneshot},
    select,
    sink::SinkExt,
    stream::{Fuse, StreamExt},
    FutureExt,
};
use log::{debug, error, info, warn};

use std::collections::HashMap;

pub struct BeeNodeBuilder {
    config: NodeConfig,
}

impl BeeNodeBuilder {
    // TODO use proper error type
    /// Finishes the build process of a new node.
    pub fn finish(self) -> Result<BeeNode, ()> {
        info!("Running v{}-{}.", BEE_VERSION, &BEE_GIT_COMMIT[0..7]);
        info!("Initializing...");

        let (network, events, shutdown) = bee_network::init(self.config.network);

        block_on(StaticPeerManager::new(self.config.peering.r#static.clone(), network.clone()).run());

        bee_tangle::init();

        info!("Reading snapshot file...");
        let snapshot_state = match block_on(LocalSnapshot::from_file(self.config.snapshot.local().file_path())) {
            Ok(local_snapshot) => {
                info!(
                "Read snapshot file from {} with index {}, {} solid entry points, {} seen milestones and {} balances.",
                Utc.timestamp(local_snapshot.metadata().timestamp() as i64, 0)
                    .to_rfc2822(),
                local_snapshot.metadata().index(),
                local_snapshot.metadata().solid_entry_points().len(),
                local_snapshot.metadata().seen_milestones().len(),
                local_snapshot.state().balances().len()
            );
                tangle().update_solid_milestone_index(local_snapshot.metadata().index().into());
                // TODO get from database
                tangle().update_snapshot_milestone_index(local_snapshot.metadata().index().into());
                tangle().add_solid_entry_point(Hash::zeros());
                for solid_entry_point in local_snapshot.metadata().solid_entry_points() {
                    tangle().add_solid_entry_point(*solid_entry_point);
                }
                for seen_milestone in local_snapshot.metadata().seen_milestones() {
                    // TODO request ?
                }
                local_snapshot.into_state()
            }
            Err(e) => {
                // TODO exit ?
                error!(
                    "Failed to read snapshot file \"{}\": {:?}.",
                    self.config.snapshot.local().file_path(),
                    e
                );
                return Err(());
            }
        };

        block_on(Protocol::init(self.config.protocol.clone(), network.clone()));

        // TODO config
        let (ledger_worker_tx, ledger_worker_rx) = mpsc::channel(1000);
        let (ledger_worker_shutdown_tx, ledger_worker_shutdown_rx) = oneshot::channel();

        spawn(LedgerWorker::new(snapshot_state.into_balances()).run(ledger_worker_rx, ledger_worker_shutdown_rx));

        info!("Initialized.");

        Ok(BeeNode {
            config: self.config,
            network,
            events: events.fuse(),
            shutdown,
            ledger: (ledger_worker_tx, ledger_worker_shutdown_tx),
            peers: HashMap::new(),
        })
    }
}

/// The main node type.
pub struct BeeNode {
    config: NodeConfig,
    // TODO those 2 fields are related; consider bundling them
    network: Network,
    events: Fuse<EventSubscriber>,
    shutdown: Shutdown,
    // TODO design proper type `Ledger`
    ledger: (mpsc::Sender<LedgerWorkerEvent>, oneshot::Sender<()>),
    // TODO design proper type `PeerList`
    peers: HashMap<EndpointId, (mpsc::Sender<Vec<u8>>, oneshot::Sender<()>)>,
}

impl BeeNode {
    /// Executes node event loop. This method is only executed after the shutdown signal has been received.
    pub fn run_loop(&mut self) {
        info!("Running.");

        let mut shutdown = shutdown_listener().fuse();

        block_on(async {
            loop {
                select! {
                    event = self.events.next() => {
                        if let Some(event) = event {
                            debug!("Received event {}.", event);

                            match event {
                                Event::EndpointAdded { epid, .. } => self.endpoint_added_handler(epid).await,
                                Event::EndpointRemoved { epid, .. } => self.endpoint_removed_handler(epid).await,
                                Event::EndpointConnected {
                                    epid, origin, address, ..
                                } => self.endpoint_connected_handler(epid, address, origin).await,
                                Event::EndpointDisconnected { epid, .. } => self.endpoint_disconnected_handler(epid).await,
                                Event::MessageReceived { epid, bytes, .. } => {
                                    self.endpoint_bytes_received_handler(epid, bytes).await
                                }
                                _ => warn!("Unsupported event {}.", event),
                            }
                        }
                    },
                    shutdown = shutdown => {
                        break;
                    }
                }
            }
        });
    }

    /// Shuts down the node.
    pub fn shutdown(self) {
        info!("Bee is shutting down...");

        block_on(self.shutdown.execute());

        info!("Shutdown complete.");
    }

    /// Returns a builder to create a node.
    pub fn build(config: NodeConfig) -> BeeNodeBuilder {
        BeeNodeBuilder { config }
    }

    async fn endpoint_added_handler(&mut self, epid: EndpointId) {
        info!("Endpoint {} has been added.", epid);

        if let Err(e) = self.network.send(Connect { epid, responder: None }).await {
            warn!("Sending Command::Connect for {} failed: {}.", epid, e);
        }
    }

    async fn endpoint_removed_handler(&mut self, epid: EndpointId) {
        info!("Endpoint {} has been removed.", epid);
    }

    async fn endpoint_connected_handler(&mut self, epid: EndpointId, address: Address, origin: Origin) {
        let (receiver_tx, receiver_shutdown_tx) = Protocol::register(epid, address, origin);

        self.peers.insert(epid, (receiver_tx, receiver_shutdown_tx));
    }

    async fn endpoint_disconnected_handler(&mut self, epid: EndpointId) {
        // TODO unregister ?
        if let Some((_, shutdown)) = self.peers.remove(&epid) {
            if let Err(e) = shutdown.send(()) {
                warn!("Sending shutdown to {} failed: {:?}.", epid, e);
            }
        }
    }

    async fn endpoint_bytes_received_handler(&mut self, epid: EndpointId, bytes: Vec<u8>) {
        if let Some(peer) = self.peers.get_mut(&epid) {
            if let Err(e) = peer.0.send(bytes).await {
                warn!("Sending PeerWorkerEvent::Message to {} failed: {}.", epid, e);
            }
        }
    }
}

// TODO return a Result
fn shutdown_listener() -> oneshot::Receiver<()> {
    let (sender, receiver) = oneshot::channel();

    spawn(async move {
        let mut rt = tokio::runtime::Runtime::new().expect("Error creating Tokio runtime.");

        rt.block_on(tokio::signal::ctrl_c()).expect("Error blocking on CTRL-C.");

        sender.send(()).expect("Error sending shutdown signal.");
    });

    receiver
}
