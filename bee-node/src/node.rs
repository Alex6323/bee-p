use bee_network::{
    Address,
    Command::Connect,
    EndpointId,
    Event,
    EventSubscriber,
    Network,
    Origin,
    Shutdown,
};
use bee_peering::{
    PeerManager,
    StaticPeerManager,
};
use bee_protocol::{
    Peer,
    PeerMetrics,
    Protocol,
    ProtocolConfBuilder,
};
use bee_snapshot::{
    SnapshotMetadata,
    SnapshotState,
};
use bee_tangle::tangle;

use std::{
    collections::HashMap,
    sync::Arc,
};

use async_std::task::block_on;
use futures::{
    channel::{
        mpsc,
        oneshot,
    },
    sink::SinkExt,
    stream::StreamExt,
};
use log::*;

pub struct Node {
    network: Network,
    shutdown: Shutdown,
    events: EventSubscriber,
    // TODO real type ?
    peers: HashMap<EndpointId, (mpsc::Sender<Vec<u8>>, oneshot::Sender<()>, Arc<Peer>)>,
    metrics: Arc<PeerMetrics>,
}

impl Node {
    pub fn new(network: Network, shutdown: Shutdown, events: EventSubscriber) -> Self {
        Self {
            network: network,
            shutdown: shutdown,
            events: events,
            peers: HashMap::new(),
            metrics: Arc::new(PeerMetrics::new()),
        }
    }

    async fn endpoint_added_handler(&mut self, epid: EndpointId) {
        info!("[Node ] Endpoint {} has been added.", epid);

        // if let Err(e) = self
        //     .network
        //     .send(Connect {
        //         epid: epid,
        //         responder: None,
        //     })
        //     .await
        // {
        //     warn!("[Node ] Sending Command::Connect for {} failed: {}.", epid, e);
        // }
    }

    async fn endpoint_removed_handler(&mut self, epid: EndpointId) {
        info!("[Node ] Endpoint {} has been removed.", epid);
    }

    async fn endpoint_connected_handler(&mut self, epid: EndpointId, address: Address, origin: Origin) {
        let peer = Arc::new(Peer::new(epid, address, origin));
        let (receiver_tx, receiver_shutdown_tx) = Protocol::register(peer.clone(), self.metrics.clone());

        self.peers.insert(epid, (receiver_tx, receiver_shutdown_tx, peer));
    }

    async fn endpoint_disconnected_handler(&mut self, epid: EndpointId) {
        //TODO unregister ?
        if let Some((_, shutdown, _)) = self.peers.remove(&epid) {
            if let Err(_) = shutdown.send(()) {
                warn!("[Node ] Sending shutdown to {} failed.", epid);
            }
        }
    }

    async fn endpoint_bytes_received_handler(&mut self, epid: EndpointId, bytes: Vec<u8>) {
        if let Some(peer) = self.peers.get_mut(&epid) {
            if let Err(e) = peer.0.send(bytes).await {
                warn!(
                    "[Node ] Sending ReceiverWorkerEvent::Message to {} failed: {}.",
                    epid, e
                );
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
        info!("[Node ] Initializing...");

        block_on(StaticPeerManager::new(self.network.clone()).run());

        bee_tangle::init();

        let protocol_conf = ProtocolConfBuilder::new().build();
        Protocol::init(self.network.clone(), protocol_conf);

        info!("[Node ] Reading snapshot metadata file...");
        // TODO conf
        match SnapshotMetadata::new("./data/mainnet.snapshot.meta") {
            Ok(snapshot_metadata) => {
                // TODO convert timestamp to date for better UX
                info!(
                    "[Node ] Snapshot metadata file read with index {}, timestamp {}, {} solid entry points and {} seen milestones.",
                    snapshot_metadata.index(),
                    snapshot_metadata.timestamp(),
                    snapshot_metadata.solid_entry_points().len(),
                    snapshot_metadata.seen_milestones().len(),
                );
                for solid_entry_point in snapshot_metadata.solid_entry_points() {
                    tangle().add_solid_entry_point(*solid_entry_point);
                }
                for seen_milestone in snapshot_metadata.seen_milestones() {
                    // TODO request ?
                }
            }
            // TODO exit ?
            Err(e) => error!("[Node ] Failed to read snapshot metadata file: {:?}.", e),
        }

        info!("[Node ] Reading snapshot state file...");
        // TODO conf
        match SnapshotState::new("./data/mainnet.snapshot.state") {
            Ok(snapshot_state) => {
                info!(
                    "[Node ] Snapshot state file read with {} entries and correct supply.",
                    snapshot_state.entries().len()
                );
                // TODO deal with entries
            }
            // TODO exit ?
            Err(e) => error!("[Node ] Failed to read snapshot state file: {:?}.", e),
        }

        info!("[Node ] Initialized.");
    }
}

#[cfg(test)]
mod tests {}
