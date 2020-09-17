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

use bee_common::shutdown_stream::ShutdownStream;
use bee_common_ext::{event::Bus, shutdown_tokio::Shutdown};
use bee_crypto::ternary::Hash;
use bee_network::{self, Command::ConnectEndpoint, EndpointId, Event, EventReceiver, Network, Origin};
use bee_peering::{PeerManager, StaticPeerManager};
use bee_protocol::{tangle, MilestoneIndex, Protocol};
use bee_snapshot::local::{download_local_snapshot, Error as LocalSnapshotReadError, LocalSnapshot};

use chrono::{offset::TimeZone, Utc};
use futures::{
    channel::{mpsc, oneshot},
    sink::SinkExt,
    stream::{Fuse, StreamExt},
};
use log::{error, info, trace, warn};
use thiserror::Error;

use std::{collections::HashMap, net::SocketAddr, sync::Arc};

type NetworkEventStream = ShutdownStream<Fuse<EventReceiver>>;

// TODO design proper type `PeerList`
type PeerList = HashMap<EndpointId, (mpsc::Sender<Vec<u8>>, oneshot::Sender<()>)>;

/// All possible node errors.
#[derive(Error, Debug)]
pub enum Error {
    /// Occurs, when there is an error while reading the snapshot file.
    #[error("Reading the snapshot file failed.")]
    LocalSnapshotReadError(LocalSnapshotReadError),

    /// Occurs, when there is an error while shutting down the node.
    #[error("Shutting down failed.")]
    ShutdownError(#[from] bee_common::shutdown::Error),

    #[error("An I/O error occurred.")]
    IoError(#[from] std::io::Error),
}

pub struct NodeBuilder {
    config: NodeConfig,
}

impl NodeBuilder {
    /// Finishes the build process of a new node.
    pub async fn finish(self) -> Result<Node, Error> {
        print_banner_and_version();

        let mut shutdown = Shutdown::new();

        let bus = Arc::new(Bus::default());

        info!("Initializing Tangle...");
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
                    local_snapshot.state().len()
                );

                tangle::tangle().update_last_solid_milestone_index(local_snapshot.metadata().index().into());

                // TODO get from database
                tangle::tangle().update_last_milestone_index(local_snapshot.metadata().index().into());

                tangle::tangle().update_snapshot_milestone_index(local_snapshot.metadata().index().into());

                // TODO index 0 ?
                tangle::tangle().add_solid_entry_point(Hash::zeros(), MilestoneIndex(0));
                for (hash, index) in local_snapshot.metadata().solid_entry_points() {
                    tangle::tangle().add_solid_entry_point(*hash, MilestoneIndex(*index));
                }

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
        let (network, events) = bee_network::init(self.config.network, &mut shutdown).await;

        info!("Starting static peer manager...");
        tokio::spawn(StaticPeerManager::new(self.config.peering.r#static.clone(), network.clone()).run());

        info!("Initializing ledger...");
        bee_ledger::whiteflag::init(
            snapshot_index,
            local_snapshot.into_state(),
            self.config.protocol.coordinator().clone(),
            bus.clone(),
            &mut shutdown,
        );

        info!("Initializing protocol...");
        Protocol::init(
            self.config.protocol.clone(),
            network.clone(),
            snapshot_timestamp,
            bus.clone(),
            &mut shutdown,
        )
        .await;

        info!("Initializing plugins...");
        plugin::init(bus, &mut shutdown);

        info!("Initialized.");
        Ok(Node {
            network,
            network_events: ShutdownStream::new(ctrl_c_listener(), events),
            shutdown,
            peers: HashMap::new(),
        })
    }
}

/// The main node type.
pub struct Node {
    // TODO those 2 fields are related; consider bundling them
    network: Network,
    network_events: NetworkEventStream,
    shutdown: Shutdown,
    peers: PeerList,
}

impl Node {
    pub async fn run(self) -> Result<(), Error> {
        info!("Running.");

        let Node {
            mut network,
            mut network_events,
            shutdown,
            mut peers,
            ..
        } = self;

        while let Some(event) = network_events.next().await {
            trace!("Received event {}.", event);

            process_event(event, &mut network, &mut peers).await;
        }

        info!("Stopping...");
        shutdown.execute().await?;

        info!("Stopped.");
        Ok(())
    }

    /// Returns a builder to create a node.
    pub fn builder(config: NodeConfig) -> NodeBuilder {
        NodeBuilder { config }
    }
}

#[inline]
async fn process_event(event: Event, network: &mut Network, peers: &mut PeerList) {
    match event {
        Event::EndpointAdded { epid, .. } => endpoint_added_handler(epid, network).await,

        Event::EndpointRemoved { epid, .. } => endpoint_removed_handler(epid).await,

        Event::EndpointConnected {
            epid,
            peer_address,
            origin,
        } => endpoint_connected_handler(epid, peer_address, origin, peers).await,

        Event::EndpointDisconnected { epid, .. } => endpoint_disconnected_handler(epid, peers).await,

        Event::MessageReceived { epid, message, .. } => endpoint_bytes_received_handler(epid, message, peers).await,
        _ => warn!("Unsupported event {}.", event),
    }
}

#[inline]
async fn endpoint_added_handler(epid: EndpointId, network: &mut Network) {
    info!("Endpoint {} has been added.", epid);

    if let Err(e) = network.send(ConnectEndpoint { epid }).await {
        warn!("Sending Command::Connect for {} failed: {}.", epid, e);
    }
}

#[inline]
async fn endpoint_removed_handler(epid: EndpointId) {
    info!("Endpoint {} has been removed.", epid);
}

#[inline]
async fn endpoint_connected_handler(epid: EndpointId, peer_address: SocketAddr, origin: Origin, peers: &mut PeerList) {
    let (receiver_tx, receiver_shutdown_tx) = Protocol::register(epid, peer_address, origin);

    peers.insert(epid, (receiver_tx, receiver_shutdown_tx));
}

#[inline]
async fn endpoint_disconnected_handler(epid: EndpointId, peers: &mut PeerList) {
    // TODO unregister ?
    if let Some((_, shutdown)) = peers.remove(&epid) {
        if let Err(e) = shutdown.send(()) {
            warn!("Sending shutdown to {} failed: {:?}.", epid, e);
        }
    }
}

#[inline]
async fn endpoint_bytes_received_handler(epid: EndpointId, bytes: Vec<u8>, peers: &mut PeerList) {
    if let Some(peer) = peers.get_mut(&epid) {
        if let Err(e) = peer.0.send(bytes).await {
            warn!("Sending PeerWorkerEvent::Message to {} failed: {}.", epid, e);
        }
    }
}

fn ctrl_c_listener() -> oneshot::Receiver<()> {
    let (sender, receiver) = oneshot::channel();

    tokio::spawn(async move {
        if let Err(e) = tokio::signal::ctrl_c().await {
            panic!("Failed to intercept CTRL-C.");
        }

        if let Err(_) = sender.send(()) {
            panic!("Failed to send the shutdown signal.")
        }
    });

    receiver
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
