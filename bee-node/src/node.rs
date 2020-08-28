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

#![warn(missing_docs)]

use crate::{
    config::NodeConfig,
    constants::{BEE_GIT_COMMIT, BEE_VERSION},
    plugin,
};

use bee_common::{shutdown::Shutdown, shutdown_stream::ShutdownStream};
use bee_common_ext::event::Bus;
use bee_crypto::ternary::Hash;
use bee_network::{self, Address, Command::Connect, EndpointId, Event, EventSubscriber, Network, Origin};
use bee_peering::{PeerManager, StaticPeerManager};
use bee_protocol::{tangle, MilestoneIndex, Protocol};
use bee_snapshot::local::{download_local_snapshot, Error as LocalSnapshotReadError, LocalSnapshot};

use async_std::task::{block_on, spawn};
use chrono::{offset::TimeZone, Utc};
use futures::{
    channel::{mpsc, oneshot},
    sink::SinkExt,
    stream::{Fuse, StreamExt},
};
use log::{debug, error, info, warn};
use thiserror::Error;

use std::{collections::HashMap, sync::Arc};

type Receiver = ShutdownStream<Fuse<EventSubscriber>>;

/// All possible node errors.
#[derive(Error, Debug)]
pub enum Error {
    /// Occurs, when there is an error while reading the snapshot file.
    #[error("Reading the snapshot file failed.")]
    LocalSnapshotReadError(LocalSnapshotReadError),

    /// Occurs, when there is an error while shutting down the node.
    #[error("Shutting down failed.")]
    ShutdownError(#[from] bee_common::shutdown::Error),
}

pub struct NodeBuilder {
    config: NodeConfig,
}

impl NodeBuilder {
    // TODO use proper error type
    /// Finishes the build process of a new node.
    pub fn finish(self) -> Result<Node, Error> {
        print_banner_and_version();

        let mut shutdown = Shutdown::new();
        let bus = Arc::new(Bus::default());

        info!("Initializing tangle...");
        tangle::init();

        // TODO handle error
        download_local_snapshot(&self.config.snapshot.local());

        info!("Reading snapshot file...");
        let local_snapshot = match LocalSnapshot::from_file(self.config.snapshot.local().file_path()) {
            Ok(local_snapshot) => {
                info!(
                    "Read snapshot file from {} with index {}, {} solid entry points, {} seen milestones and \
                    {} balances.",
                    Utc.timestamp(local_snapshot.metadata().timestamp() as i64, 0)
                        .to_rfc2822(),
                    local_snapshot.metadata().index(),
                    local_snapshot.metadata().solid_entry_points().len(),
                    local_snapshot.metadata().seen_milestones().len(),
                    local_snapshot.state().balances().len()
                );

                // add solid entry points first before updating to indices below
                // TODO index 0 ?
                tangle::tangle().add_solid_entry_point(Hash::zeros(), MilestoneIndex(0));
                for (hash, index) in local_snapshot.metadata().solid_entry_points() {
                    tangle::tangle().add_solid_entry_point(*hash, MilestoneIndex(*index));
                }

                tangle::tangle().update_last_solid_milestone_index(local_snapshot.metadata().index().into());

                // TODO get from database
                tangle::tangle().update_last_milestone_index(local_snapshot.metadata().index().into());

                tangle::tangle().update_snapshot_milestone_index(local_snapshot.metadata().index().into());

                for _seen_milestone in local_snapshot.metadata().seen_milestones() {
                    // TODO request ?
                }

                local_snapshot
            }
            Err(e) => {
                error!(
                    "Failed to read snapshot file \"{}\": {:?}.",
                    self.config.snapshot.local().file_path(),
                    e
                );
                return Err(Error::LocalSnapshotReadError(e));
            }
        };

        // TODO this is temporary
        let snapshot_index = local_snapshot.metadata().index();
        let snapshot_timestamp = local_snapshot.metadata().timestamp();

        info!("Initializing network...");
        let (network, events) = bee_network::init(self.config.network, &mut shutdown);

        info!("Starting static peer manager...");
        spawn(StaticPeerManager::new(self.config.peering.r#static.clone(), network.clone()).run());

        info!("Initializing ledger...");
        bee_ledger::whiteflag::init(
            snapshot_index,
            local_snapshot.into_state().into_balances(),
            self.config.protocol.coordinator().clone(),
            bus.clone(),
            &mut shutdown,
        );

        block_on(Protocol::init(
            self.config.protocol.clone(),
            network.clone(),
            snapshot_timestamp,
            bus.clone(),
            &mut shutdown,
        ));

        info!("Initializing plugins...");

        plugin::init(bus, &mut shutdown);

        info!("Initialized.");

        let (sender, receiver) = shutdown_listener();

        Ok(Node {
            sender,
            network,
            receiver: ShutdownStream::new(receiver, events),
            shutdown,
            peers: HashMap::new(),
        })
    }
}

/// The main node type.
pub struct Node {
    // TODO temporary to keep it alive
    sender: oneshot::Sender<()>,
    // TODO those 2 fields are related; consider bundling them
    network: Network,
    receiver: Receiver,
    shutdown: Shutdown,
    // TODO design proper type `PeerList`
    peers: HashMap<EndpointId, (mpsc::Sender<Vec<u8>>, oneshot::Sender<()>)>,
}

impl Node {
    /// Executes node event loop. This method is only executed after the shutdown signal has been received.
    pub fn run_loop(&mut self) {
        info!("Running.");

        block_on(async {
            while let Some(event) = self.receiver.next().await {
                debug!("Received event {}.", event);

                self.handle_event(event).await;
            }
        });

        info!("Stopped.");
    }

    #[inline]
    async fn handle_event(&mut self, event: Event) {
        match event {
            Event::EndpointAdded { epid, .. } => self.endpoint_added_handler(epid).await,
            Event::EndpointRemoved { epid, .. } => self.endpoint_removed_handler(epid),
            Event::EndpointConnected {
                epid, origin, address, ..
            } => self.endpoint_connected_handler(epid, address, origin),
            Event::EndpointDisconnected { epid, .. } => self.endpoint_disconnected_handler(epid),
            Event::MessageReceived { epid, bytes, .. } => self.endpoint_bytes_received_handler(epid, bytes).await,
            _ => warn!("Unsupported event {}.", event),
        }
    }

    /// Shuts down the node.
    pub fn shutdown(self) -> Result<(), Error> {
        info!("Stopping...");

        block_on(self.shutdown.execute())?;

        info!("Shutdown complete.");

        Ok(())
    }

    /// Returns a builder to create a node.
    pub fn build(config: NodeConfig) -> NodeBuilder {
        NodeBuilder { config }
    }

    async fn endpoint_added_handler(&mut self, epid: EndpointId) {
        info!("Endpoint {} has been added.", epid);

        if let Err(e) = self.network.send(Connect { epid, responder: None }).await {
            warn!("Sending Command::Connect for {} failed: {}.", epid, e);
        }
    }

    fn endpoint_removed_handler(&mut self, epid: EndpointId) {
        info!("Endpoint {} has been removed.", epid);
    }

    fn endpoint_connected_handler(&mut self, epid: EndpointId, address: Address, origin: Origin) {
        let (receiver_tx, receiver_shutdown_tx) = Protocol::register(epid, address, origin);

        self.peers.insert(epid, (receiver_tx, receiver_shutdown_tx));
    }

    fn endpoint_disconnected_handler(&mut self, epid: EndpointId) {
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
fn shutdown_listener() -> (oneshot::Sender<()>, oneshot::Receiver<()>) {
    let (sender, receiver) = oneshot::channel();

    // TODO temporarily disabled because conflicting with receiving messages

    // spawn(async move {
    //     let mut rt = tokio::runtime::Runtime::new().expect("Error creating Tokio runtime.");
    //
    //     rt.block_on(tokio::signal::ctrl_c()).expect("Error blocking on CTRL-C.");
    //
    //     sender.send(()).expect("Error sending shutdown signal.");
    // });

    // TODO temporarily returns sender as well
    (sender, receiver)
}

fn print_banner_and_version() {
    let commit = if BEE_GIT_COMMIT.is_empty() {
        "".to_owned()
    } else {
        "-".to_owned() + &BEE_GIT_COMMIT[0..7]
    };
    println!(
        "\n{}\n   v{}{}\n",
        " ██████╗░███████╗███████╗
 ██╔══██╗██╔════╝██╔════╝
 ██████╦╝█████╗░░█████╗░░
 ██╔══██╗██╔══╝░░██╔══╝░░
 ██████╦╝███████╗███████╗
 ╚═════╝░╚══════╝╚══════╝",
        BEE_VERSION,
        commit,
    );
}
