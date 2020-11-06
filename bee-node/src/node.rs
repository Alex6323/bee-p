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
    banner::print_banner_and_version, config::NodeConfig, inner::BeeNode, plugin, storage::Backend,
    version_checker::VersionCheckerWorker,
};

use bee_common::shutdown_stream::ShutdownStream;
use bee_common_ext::{
    node::{Node as _, NodeBuilder as _},
    shutdown_tokio::Shutdown,
};
use bee_network::{self, Command::DialPeer, Event, Multiaddr, Network, Origin, PeerId};
use bee_peering::{ManualPeerManager, PeerManager};
use bee_protocol::Protocol;

use futures::{
    channel::oneshot,
    stream::{Fuse, StreamExt},
};
use log::{error, info, trace, warn};
use thiserror::Error;
use tokio::spawn;

use std::{collections::HashMap, sync::Arc};

type NetworkEventStream = ShutdownStream<Fuse<flume::r#async::RecvStream<'static, Event>>>;

// TODO design proper type `PeerList`
type PeerList = HashMap<PeerId, (flume::Sender<Vec<u8>>, oneshot::Sender<()>)>;

/// All possible node errors.
#[derive(Error, Debug)]
pub enum Error {
    /// Occurs, when there is an error while reading the snapshot file.
    #[error("Reading snapshot file failed.")]
    SnapshotError(bee_snapshot::Error),

    /// Occurs, when there is an error while shutting down the node.
    #[error("Shutting down failed.")]
    ShutdownError(#[from] bee_common::shutdown::Error),
}

pub struct NodeBuilder<B: Backend> {
    config: NodeConfig<B>,
}

impl<B: Backend> NodeBuilder<B> {
    /// Finishes the build process of a new node.
    pub async fn finish(self) -> Result<Node<B>, Error> {
        print_banner_and_version();

        let node_builder = BeeNode::<B>::build();

        let mut shutdown = Shutdown::new();

        let (mut node_builder, snapshot) = bee_snapshot::init::<BeeNode<B>>(&self.config.snapshot, node_builder)
            .await
            .map_err(Error::SnapshotError)?;

        info!("Initializing network...");
        let (network, events) = bee_network::init(self.config.network.clone(), &mut shutdown).await;

        info!("Starting manual peer manager...");
        spawn(ManualPeerManager::new(self.config.peering.manual.clone(), network.clone()).run());

        // info!("Initializing ledger...");
        // node_builder = bee_ledger::whiteflag::init::<BeeNode<B>>(
        //     snapshot_metadata.index(),
        //     snapshot_state.into(),
        //     self.config.protocol.coordinator().clone(),
        //     node_builder,
        //     bus.clone(),
        // );

        info!("Initializing protocol...");
        node_builder = Protocol::init::<BeeNode<B>>(
            self.config.protocol.clone(),
            self.config.database.clone(),
            network.clone(),
            &snapshot,
            node_builder,
        );

        info!("Initializing plugins...");
        // plugin::init(bus.clone());

        node_builder = node_builder.with_worker::<VersionCheckerWorker>();

        let bee_node = node_builder.finish().await;

        info!("Registering events...");
        bee_snapshot::events(&bee_node);
        // bee_ledger::whiteflag::events(&bee_node, bus.clone());
        Protocol::events(&bee_node, self.config.protocol.clone());

        info!("Initialized.");
        Ok(Node {
            config: self.config,
            tmp_node: bee_node,
            network,
            network_events: ShutdownStream::new(ctrl_c_listener(), events.into_stream()),
            shutdown,
            peers: HashMap::new(),
        })
    }
}

/// The main node type.
pub struct Node<B: Backend> {
    tmp_node: BeeNode<B>,
    // TODO those 2 fields are related; consider bundling them
    network: Network,
    network_events: NetworkEventStream,
    #[allow(dead_code)]
    shutdown: Shutdown,
    peers: PeerList,
    config: NodeConfig<B>,
}
impl<B: Backend> Node<B> {
    #[allow(missing_docs)]
    pub async fn run(mut self) -> Result<(), Error> {
        info!("Running.");

        while let Some(event) = self.network_events.next().await {
            trace!("Received event {}.", event);

            self.process_event(event);
        }

        info!("Stopping...");

        for (_, (_, shutdown)) in self.peers.into_iter() {
            // TODO: Should we handle this error?
            let _ = shutdown.send(());
        }

        self.tmp_node.stop().await.expect("Failed to properly stop node");

        info!("Stopped.");

        Ok(())
    }

    /// Returns a builder to create a node.
    pub fn builder(config: NodeConfig<B>) -> NodeBuilder<B> {
        NodeBuilder { config }
    }

    #[inline]
    fn process_event(&mut self, event: Event) {
        match event {
            Event::EndpointAdded { address } => self.endpoint_added_handler(address),

            Event::EndpointRemoved { address } => self.endpoint_removed_handler(address),

            Event::PeerConnected {
                id,
                endpoint_address,
                origin,
            } => self.peer_connected_handler(id, endpoint_address, origin),

            Event::PeerDisconnected { id } => self.peer_disconnected_handler(id),

            Event::MessageReceived { peer_id, message, .. } => self.peer_message_received_handler(peer_id, message),
            _ => warn!("Unsupported event {}.", event),
        }
    }

    #[inline]
    fn endpoint_added_handler(&self, address: Multiaddr) {
        info!("Endpoint {} has been added.", address);

        if let Err(e) = self.network.unbounded_send(DialPeer {
            endpoint_address: address.clone(),
        }) {
            warn!("Sending Command::DialPeer for {} failed: {}.", address, e);
        }
    }

    #[inline]
    fn endpoint_removed_handler(&self, address: Multiaddr) {
        info!("Endpoint {} has been removed.", address);
    }

    #[inline]
    fn peer_connected_handler(&mut self, id: PeerId, endpoint_address: Multiaddr, origin: Origin) {
        let (receiver_tx, receiver_shutdown_tx) = Protocol::register(
            &self.tmp_node,
            &self.config.protocol,
            id.clone(),
            endpoint_address,
            origin,
        );

        self.peers.insert(id, (receiver_tx, receiver_shutdown_tx));
    }

    #[inline]
    fn peer_disconnected_handler(&mut self, id: PeerId) {
        // TODO unregister ?
        if let Some((_, shutdown)) = self.peers.remove(&id) {
            if let Err(e) = shutdown.send(()) {
                warn!("Sending shutdown to {} failed: {:?}.", id, e);
            }
        }
    }

    #[inline]
    fn peer_message_received_handler(&mut self, peer_id: PeerId, message: Vec<u8>) {
        if let Some(peer) = self.peers.get_mut(&peer_id) {
            if let Err(e) = peer.0.send(message) {
                warn!("Sending PeerWorkerEvent::Message to {} failed: {}.", peer_id, e);
            }
        }
    }
}

fn ctrl_c_listener() -> oneshot::Receiver<()> {
    let (sender, receiver) = oneshot::channel();

    tokio::spawn(async move {
        if let Err(e) = tokio::signal::ctrl_c().await {
            panic!("Failed to intercept CTRL-C: {:?}.", e);
        }

        if let Err(e) = sender.send(()) {
            panic!("Failed to send the shutdown signal: {:?}.", e);
        }
    });

    receiver
}
