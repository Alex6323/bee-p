use bee_network::{
    Command::Connect,
    EndpointId,
    Event,
    EventSubscriber,
    Network,
    Role,
    Shutdown,
};
use bee_peering::{
    PeerManager,
    StaticPeerManager,
};
use bee_protocol::{
    Handshake,
    Heartbeat,
    MilestoneRequest,
    MilestoneValidatorWorker,
    MilestoneValidatorWorkerEvent,
    Peer,
    PeerMetrics,
    ReceiverWorker,
    ReceiverWorkerEvent,
    RequesterWorker,
    RequesterWorkerEvent,
    ResponderWorker,
    ResponderWorkerEvent,
    SenderContext,
    SenderRegistry,
    SenderWorker,
    TransactionBroadcast,
    TransactionRequest,
    TransactionWorker,
    TransactionWorkerEvent,
    HANDSHAKE_SEND_BOUND,
    HEARTBEAT_SEND_BOUND,
    MILESTONE_REQUEST_SEND_BOUND,
    TRANSACTION_BROADCAST_SEND_BOUND,
    TRANSACTION_REQUEST_SEND_BOUND,
};
use bee_snapshot::{
    SnapshotMetadata,
    SnapshotState,
};

use std::{
    collections::HashMap,
    sync::Arc,
};

use async_std::task::{
    block_on,
    spawn,
};
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
    peers: HashMap<EndpointId, (mpsc::Sender<ReceiverWorkerEvent>, oneshot::Sender<()>, Arc<Peer>)>,
    metrics: Arc<PeerMetrics>,
    transaction_worker_sender: Option<mpsc::Sender<TransactionWorkerEvent>>,
    responder_worker_sender: Option<mpsc::Sender<ResponderWorkerEvent>>,
    requester_worker_sender: Option<mpsc::Sender<RequesterWorkerEvent>>,
    milestone_validator_worker_sender: Option<mpsc::Sender<MilestoneValidatorWorkerEvent>>,
}

impl Node {
    pub fn new(network: Network, shutdown: Shutdown, events: EventSubscriber) -> Self {
        Self {
            network: network,
            shutdown: shutdown,
            events: events,
            peers: HashMap::new(),
            metrics: Arc::new(PeerMetrics::new()),
            transaction_worker_sender: None,
            responder_worker_sender: None,
            requester_worker_sender: None,
            milestone_validator_worker_sender: None,
        }
    }

    async fn endpoint_added_handler(&mut self, epid: EndpointId) {
        info!("[Node ] Endpoint {} has been added.", epid);

        if let Err(e) = self
            .network
            .send(Connect {
                epid: epid,
                responder: None,
            })
            .await
        {
            warn!("[Node ] Sending Command::Connect for {} failed: {}.", epid, e);
        }
    }

    async fn endpoint_removed_handler(&mut self, epid: EndpointId) {
        info!("[Node ] Endpoint {} has been removed.", epid);
    }

    async fn endpoint_connected_handler(&mut self, epid: EndpointId, role: Role) {
        // TODO conf
        // ReceiverWorker channels
        let (receiver_tx, receiver_rx) = mpsc::channel(1000);
        let (receiver_shutdown_tx, receiver_shutdown_rx) = oneshot::channel();

        // SenderWorker Handshake channels
        let (handshake_tx, handshake_rx) = mpsc::channel(HANDSHAKE_SEND_BOUND);
        let (handshake_shutdown_tx, handshake_shutdown_rx) = oneshot::channel();

        // SenderWorker MilestoneRequest channels
        let (milestone_request_tx, milestone_request_rx) = mpsc::channel(MILESTONE_REQUEST_SEND_BOUND);
        let (milestone_request_shutdown_tx, milestone_request_shutdown_rx) = oneshot::channel();

        // SenderWorker TransactionBroadcast channels
        let (transaction_broadcast_tx, transaction_broadcast_rx) = mpsc::channel(TRANSACTION_BROADCAST_SEND_BOUND);
        let (transaction_broadcast_shutdown_tx, transaction_broadcast_shutdown_rx) = oneshot::channel();

        // SenderWorker TransactionRequest channels
        let (transaction_request_tx, transaction_request_rx) = mpsc::channel(TRANSACTION_REQUEST_SEND_BOUND);
        let (transaction_request_shutdown_tx, transaction_request_shutdown_rx) = oneshot::channel();

        // SenderWorker Heartbeat channels
        let (heartbeat_tx, heartbeat_rx) = mpsc::channel(HEARTBEAT_SEND_BOUND);
        let (heartbeat_shutdown_tx, heartbeat_shutdown_rx) = oneshot::channel();

        let context = SenderContext::new(
            (handshake_tx, handshake_shutdown_tx),
            (milestone_request_tx, milestone_request_shutdown_tx),
            (transaction_broadcast_tx, transaction_broadcast_shutdown_tx),
            (transaction_request_tx, transaction_request_shutdown_tx),
            (heartbeat_tx, heartbeat_shutdown_tx),
        );

        let peer = Arc::new(Peer::new(epid, role));

        self.peers
            .insert(epid, (receiver_tx, receiver_shutdown_tx, peer.clone()));
        SenderRegistry::insert(epid, context).await;

        spawn(
            ReceiverWorker::new(
                peer.clone(),
                self.metrics.clone(),
                self.transaction_worker_sender.as_ref().unwrap().clone(),
                self.responder_worker_sender.as_ref().unwrap().clone(),
            )
            .run(receiver_rx, receiver_shutdown_rx),
        );

        spawn(
            SenderWorker::<Handshake>::new(peer.clone(), self.metrics.clone(), self.network.clone())
                .run(handshake_rx, handshake_shutdown_rx),
        );
        spawn(
            SenderWorker::<MilestoneRequest>::new(peer.clone(), self.metrics.clone(), self.network.clone())
                .run(milestone_request_rx, milestone_request_shutdown_rx),
        );
        spawn(
            SenderWorker::<TransactionBroadcast>::new(peer.clone(), self.metrics.clone(), self.network.clone())
                .run(transaction_broadcast_rx, transaction_broadcast_shutdown_rx),
        );
        spawn(
            SenderWorker::<TransactionRequest>::new(peer.clone(), self.metrics.clone(), self.network.clone())
                .run(transaction_request_rx, transaction_request_shutdown_rx),
        );
        spawn(
            SenderWorker::<Heartbeat>::new(peer.clone(), self.metrics.clone(), self.network.clone())
                .run(heartbeat_rx, heartbeat_shutdown_rx),
        );
    }

    async fn endpoint_disconnected_handler(&mut self, epid: EndpointId) {
        if let Some((_, shutdown, _)) = self.peers.remove(&epid) {
            if let Err(_) = shutdown.send(()) {
                warn!("[Node ] Sending shutdown to {} failed.", epid);
            }
        }
        if let Some(context) = SenderRegistry::remove(&epid).await {
            context.shutdown();
        }
    }

    async fn endpoint_bytes_received_handler(&mut self, epid: EndpointId, bytes: Vec<u8>) {
        if let Some(peer) = self.peers.get_mut(&epid) {
            if let Err(e) = peer.0.send(ReceiverWorkerEvent::Message(bytes)).await {
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
                Event::EndpointConnected { epid, role, .. } => self.endpoint_connected_handler(epid, role).await,
                Event::EndpointDisconnected { epid, .. } => self.endpoint_disconnected_handler(epid).await,
                Event::BytesReceived { epid, bytes, .. } => self.endpoint_bytes_received_handler(epid, bytes).await,
                _ => warn!("[Node ] Unsupported event {}.", event),
            }
        }
    }

    pub async fn init(&mut self) {
        info!("[Node ] Initializing...");

        block_on(StaticPeerManager::new(self.network.clone()).run());

        SenderRegistry::init();

        // TODO conf
        let (milestone_validator_worker_sender, milestone_validator_worker_receiver) = mpsc::channel(1000);
        self.milestone_validator_worker_sender = Some(milestone_validator_worker_sender);
        spawn(MilestoneValidatorWorker::new(milestone_validator_worker_receiver).run());

        // TODO conf
        let (transaction_worker_sender, transaction_worker_receiver) = mpsc::channel(1000);
        self.transaction_worker_sender = Some(transaction_worker_sender);
        spawn(TransactionWorker::new(transaction_worker_receiver).run());

        // TODO conf
        let (responder_worker_sender, responder_worker_receiver) = mpsc::channel(1000);
        self.responder_worker_sender = Some(responder_worker_sender);
        spawn(ResponderWorker::new(responder_worker_receiver).run());

        // TODO conf
        let (requester_worker_sender, requester_worker_receiver) = mpsc::channel(1000);
        self.requester_worker_sender = Some(requester_worker_sender);
        spawn(RequesterWorker::new(requester_worker_receiver).run());

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
                // TODO deal with SEPs
                // TODO deal with SMs
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
