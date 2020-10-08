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

use crate::{banner::print_banner_and_version, config::NodeConfig, inner::BeeNode, plugin};

use bee_common::shutdown_stream::ShutdownStream;
use bee_common_ext::{
    event::Bus,
    node::{Node as _, NodeBuilder as _},
    shutdown_tokio::Shutdown,
};
use bee_network::{self, Command::ConnectEndpoint, EndpointId, Event, Network, Origin};
use bee_peering::{ManualPeerManager, PeerManager};
use bee_protocol::Protocol;
use bee_storage::storage::Backend;

use futures::{
    channel::oneshot,
    stream::{Fuse, StreamExt},
};
use log::{error, info, trace, warn};
use thiserror::Error;
use tokio::spawn;

use std::{collections::HashMap, net::SocketAddr, sync::Arc};

type NetworkEventStream = ShutdownStream<Fuse<flume::r#async::RecvStream<'static, Event>>>;

// TODO design proper type `PeerList`
type PeerList = HashMap<EndpointId, (flume::Sender<Vec<u8>>, oneshot::Sender<()>)>;

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

        let bus = Arc::new(Bus::default());

        // TODO temporary
        let (mut node_builder, snapshot_state, snapshot_metadata) =
            bee_snapshot::init::<BeeNode<B>>(&self.config.snapshot, node_builder)
                .await
                .map_err(Error::SnapshotError)?;

        info!("Initializing network...");
        let (network, events) = bee_network::init(self.config.network, &mut shutdown).await;

        info!("Starting manual peer manager...");
        spawn(ManualPeerManager::new(self.config.peering.manual.clone(), network.clone()).run());

        info!("Initializing ledger...");
        node_builder = bee_ledger::whiteflag::init::<BeeNode<B>>(
            snapshot_metadata.index(),
            snapshot_state.into(),
            self.config.protocol.coordinator().clone(),
            node_builder,
            bus.clone(),
        );

        info!("Initializing protocol...");
        node_builder = Protocol::init::<BeeNode<B>>(
            self.config.protocol,
            self.config.database,
            network.clone(),
            snapshot_metadata,
            node_builder,
            bus.clone(),
        )
        .await;

        info!("Initializing plugins...");
        plugin::init(bus.clone());

        let bee_node = node_builder.finish().await;

        info!("Registering events...");
        bee_snapshot::events(&bee_node, bus.clone());
        bee_ledger::whiteflag::events(&bee_node, bus.clone());
        Protocol::events(&bee_node, bus.clone());

        info!("Initialized.");
        Ok(Node {
            tmp_node: bee_node,
            network,
            network_events: ShutdownStream::new(ctrl_c_listener(), events.into_stream()),
            shutdown,
            peers: HashMap::new(),
        })
    }
}

/// The main node type.
pub struct Node<B> {
    tmp_node: BeeNode<B>,
    // TODO those 2 fields are related; consider bundling them
    network: Network,
    network_events: NetworkEventStream,
    #[allow(dead_code)]
    shutdown: Shutdown,
    peers: PeerList,
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
            Event::EndpointAdded { epid, .. } => self.endpoint_added_handler(epid),

            Event::EndpointRemoved { epid, .. } => self.endpoint_removed_handler(epid),

            Event::EndpointConnected {
                epid,
                peer_address,
                origin,
            } => self.endpoint_connected_handler(epid, peer_address, origin),

            Event::EndpointDisconnected { epid, .. } => self.endpoint_disconnected_handler(epid),

            Event::MessageReceived { epid, message, .. } => self.endpoint_bytes_received_handler(epid, message),
            _ => warn!("Unsupported event {}.", event),
        }
    }

    #[inline]
    fn endpoint_added_handler(&self, epid: EndpointId) {
        info!("Endpoint {} has been added.", epid);

        if let Err(e) = self.network.unbounded_send(ConnectEndpoint { epid }) {
            warn!("Sending Command::Connect for {} failed: {}.", epid, e);
        }
    }

    #[inline]
    fn endpoint_removed_handler(&self, epid: EndpointId) {
        info!("Endpoint {} has been removed.", epid);
    }

    #[inline]
    fn endpoint_connected_handler(&mut self, epid: EndpointId, peer_address: SocketAddr, origin: Origin) {
        let (receiver_tx, receiver_shutdown_tx) = Protocol::register(&self.tmp_node, epid, peer_address, origin);

        self.peers.insert(epid, (receiver_tx, receiver_shutdown_tx));
    }

    #[inline]
    fn endpoint_disconnected_handler(&mut self, epid: EndpointId) {
        // TODO unregister ?
        if let Some((_, shutdown)) = self.peers.remove(&epid) {
            if let Err(e) = shutdown.send(()) {
                warn!("Sending shutdown to {} failed: {:?}.", epid, e);
            }
        }
    }

    #[inline]
    fn endpoint_bytes_received_handler(&mut self, epid: EndpointId, bytes: Vec<u8>) {
        if let Some(peer) = self.peers.get_mut(&epid) {
            if let Err(e) = peer.0.send(bytes) {
                warn!("Sending PeerWorkerEvent::Message to {} failed: {}.", epid, e);
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
