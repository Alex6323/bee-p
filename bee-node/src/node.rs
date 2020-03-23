use bee_network::Command::Connect;
use bee_network::{
    EndpointId,
    Event,
    EventSubscriber,
    Network,
    Shutdown,
};
use bee_peering::{
    PeerManager,
    StaticPeerManager,
};
use bee_protocol::{
    NodeMetrics,
    ReceiverWorker,
    ReceiverWorkerEvent,
    RequesterWorker,
    RequesterWorkerEvent,
    ResponderWorker,
    ResponderWorkerEvent,
    TransactionWorker,
    TransactionWorkerEvent,
};

use std::collections::HashMap;

use async_std::task::{
    block_on,
    spawn,
};
use futures::channel::mpsc::{
    channel,
    Sender,
};
use futures::sink::SinkExt;
use futures::stream::StreamExt;
use log::*;

pub struct Node {
    network: Network,
    shutdown: Shutdown,
    events: EventSubscriber,
    // TODO thread-safety
    neighbors: HashMap<EndpointId, Sender<ReceiverWorkerEvent>>,
    transaction_worker_sender: Option<Sender<TransactionWorkerEvent>>,
    responder_worker_sender: Option<Sender<ResponderWorkerEvent>>,
    requester_worker_sender: Option<Sender<RequesterWorkerEvent>>,
    metrics: NodeMetrics,
}

impl Node {
    pub fn new(network: Network, shutdown: Shutdown, events: EventSubscriber) -> Self {
        Self {
            network: network,
            shutdown: shutdown,
            events: events,
            neighbors: HashMap::new(),
            transaction_worker_sender: None,
            responder_worker_sender: None,
            requester_worker_sender: None,
            metrics: NodeMetrics::default(),
        }
    }

    async fn endpoint_added_handler(&mut self, epid: EndpointId) {
        let (sender, receiver) = channel(1000);

        self.neighbors.insert(epid, sender);

        spawn(
            ReceiverWorker::new(
                epid,
                self.network.clone(),
                receiver,
                self.transaction_worker_sender.as_ref().unwrap().clone(),
                self.responder_worker_sender.as_ref().unwrap().clone(),
            )
            .run(),
        );

        if let Err(e) = self
            .network
            .send(Connect {
                epid: epid,
                responder: None,
            })
            .await
        {
            warn!("[Node ] Sending Command::Connect for {} failed: {}", epid, e);
        }
    }

    async fn endpoint_removed_handler(&mut self, epid: EndpointId) {
        if let Some(sender) = self.neighbors.get_mut(&epid) {
            if let Err(e) = sender.send(ReceiverWorkerEvent::Removed).await {
                warn!("[Node ] Sending ReceiverWorkerEvent::Removed to {} failed: {}", epid, e);
            }
            self.neighbors.remove(&epid);
        }
    }

    async fn endpoint_connected_handler(&mut self, epid: EndpointId) {
        if let Some(sender) = self.neighbors.get_mut(&epid) {
            if let Err(e) = sender.send(ReceiverWorkerEvent::Connected).await {
                warn!(
                    "[Node ] Sending ReceiverWorkerEvent::Connected to {} failed: {}",
                    epid, e
                );
            }
        }
    }

    async fn endpoint_disconnected_handler(&mut self, epid: EndpointId) {
        if let Some(sender) = self.neighbors.get_mut(&epid) {
            if let Err(e) = sender.send(ReceiverWorkerEvent::Disconnected).await {
                warn!(
                    "[Node ] Sending ReceiverWorkerEvent::Disconnected to {} failed: {}",
                    epid, e
                );
            }
        }
    }

    async fn endpoint_bytes_received_handler(&mut self, epid: EndpointId, bytes: Vec<u8>) {
        if let Some(sender) = self.neighbors.get_mut(&epid) {
            if let Err(e) = sender.send(ReceiverWorkerEvent::Message(bytes)).await {
                warn!("[Node ] Sending ReceiverWorkerEvent::Message to {} failed: {}", epid, e);
            }
        }
    }

    pub async fn run(mut self) {
        info!("[Node ] Starting actor");
        while let Some(event) = self.events.next().await {
            debug!("[Node ] Received event {}", event);
            match event {
                Event::EndpointAdded { epid, .. } => self.endpoint_added_handler(epid).await,
                Event::EndpointRemoved { epid, .. } => self.endpoint_removed_handler(epid).await,
                Event::EndpointConnected { epid, .. } => self.endpoint_connected_handler(epid).await,
                Event::EndpointDisconnected { epid, .. } => self.endpoint_disconnected_handler(epid).await,
                Event::BytesReceived { epid, bytes, .. } => self.endpoint_bytes_received_handler(epid, bytes).await,
                _ => (),
            }
        }
    }

    pub async fn init(&mut self) {
        info!("[Node ] Initializing...");
        block_on(StaticPeerManager::new(self.network.clone()).run());

        let (transaction_worker_sender, transaction_worker_receiver) = channel(1000);
        self.transaction_worker_sender = Some(transaction_worker_sender);
        spawn(TransactionWorker::new(transaction_worker_receiver).run());

        let (responder_worker_sender, responder_worker_receiver) = channel(1000);
        self.responder_worker_sender = Some(responder_worker_sender);
        spawn(ResponderWorker::new(self.network.clone(), responder_worker_receiver).run());

        let (requester_worker_sender, requester_worker_receiver) = channel(1000);
        self.requester_worker_sender = Some(requester_worker_sender);
        spawn(RequesterWorker::new(self.network.clone(), requester_worker_receiver).run());

        info!("[Node ] Initialized");
    }
}

#[cfg(test)]
mod tests {}
