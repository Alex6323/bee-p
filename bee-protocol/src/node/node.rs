use crate::message::{Handshake, Heartbeat, MilestoneRequest, TransactionBroadcast, TransactionRequest};
use crate::neighbor::Neighbor;
use crate::node::NodeMetrics;
use crate::worker::{
    ReceiverWorker, ReceiverWorkerEvent, RequesterWorker, RequesterWorkerEvent, ResponderWorker, ResponderWorkerEvent,
    TransactionWorker,
};

use bee_network::Command::Connect;
use bee_network::{EndpointId, Event, EventSubscriber, Network, Shutdown};
use bee_peering::{PeerManager, StaticPeerManager};

use std::collections::HashMap;

use async_std::task::{block_on, spawn};
use futures::channel::mpsc::{channel, SendError, Sender};
use futures::sink::SinkExt;
use futures::stream::StreamExt;
use log::*;

pub struct Node {
    network: Network,
    shutdown: Shutdown,
    events: EventSubscriber,
    // TODO thread-safety
    neighbors: HashMap<EndpointId, Sender<ReceiverWorkerEvent>>,
    transaction_worker_sender: Option<Sender<TransactionBroadcast>>,
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

        self.network
            .send(Connect {
                epid: epid,
                responder: None,
            })
            .await;
    }

    async fn endpoint_removed_handler(&mut self, epid: EndpointId) {
        if let Some(sender) = self.neighbors.get_mut(&epid) {
            sender.send(ReceiverWorkerEvent::Removed).await;
            self.neighbors.remove(&epid);
        }
    }

    async fn endpoint_connected_handler(&mut self, epid: EndpointId) {
        if let Some(sender) = self.neighbors.get_mut(&epid) {
            sender.send(ReceiverWorkerEvent::Connected).await;
        }
    }

    async fn endpoint_disconnected_handler(&mut self, epid: EndpointId) {
        if let Some(sender) = self.neighbors.get_mut(&epid) {
            sender.send(ReceiverWorkerEvent::Disconnected).await;
        }
    }

    async fn endpoint_bytes_received_handler(&mut self, epid: EndpointId, bytes: Vec<u8>) {
        if let Some(sender) = self.neighbors.get_mut(&epid) {
            sender.send(ReceiverWorkerEvent::Message(bytes)).await;
        }
    }

    pub async fn run(mut self) {
        info!("[Node ] Starting actor");
        while let Some(event) = self.events.next().await {
            debug!("[Node ] Received event {:?}", event);
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

    async fn send_handshake(&self, neighbor: &mut Neighbor, handshake: Handshake) -> Result<(), SendError> {
        let res = neighbor.senders.handshake.send(handshake).await;

        if res.is_ok() {
            neighbor.metrics.handshake_sent_inc();
            self.metrics.handshake_sent_inc();
        }

        res
    }

    async fn send_milestone_request(
        &self,
        neighbor: &mut Neighbor,
        milestone_request: MilestoneRequest,
    ) -> Result<(), SendError> {
        let res = neighbor.senders.milestone_request.send(milestone_request).await;

        if res.is_ok() {
            neighbor.metrics.milestone_request_sent_inc();
            self.metrics.milestone_request_sent_inc();
        }

        res
    }

    async fn send_transaction_broadcast(
        &self,
        neighbor: &mut Neighbor,
        transaction_broadcast: TransactionBroadcast,
    ) -> Result<(), SendError> {
        let res = neighbor.senders.transaction_broadcast.send(transaction_broadcast).await;

        if res.is_ok() {
            neighbor.metrics.transaction_broadcast_sent_inc();
            self.metrics.transaction_broadcast_sent_inc();
        }

        res
    }

    async fn send_transaction_request(
        &self,
        neighbor: &mut Neighbor,
        transaction_request: TransactionRequest,
    ) -> Result<(), SendError> {
        let res = neighbor.senders.transaction_request.send(transaction_request).await;

        if res.is_ok() {
            neighbor.metrics.transaction_request_sent_inc();
            self.metrics.transaction_request_sent_inc();
        }

        res
    }

    async fn send_heartbeat(&self, neighbor: &mut Neighbor, heartbeat: Heartbeat) -> Result<(), SendError> {
        let res = neighbor.senders.heartbeat.send(heartbeat).await;

        if res.is_ok() {
            neighbor.metrics.heartbeat_sent_inc();
            self.metrics.heartbeat_sent_inc();
        }

        res
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use futures::stream::{Stream, StreamExt};

    // #[test]
    // fn send_handshake_test() {
    //     let node = Node::new();
    //     let mut channels = NeighborChannels::new();
    //     let mut neighbor = Neighbor::new(channels.senders);
    //
    //     assert_eq!(node.metrics.handshake_sent(), 0);
    //     assert_eq!(neighbor.metrics.handshake_sent(), 0);
    //
    //     assert!(channels.receivers.handshake.try_next().is_err());
    //     assert!(block_on(node.send_handshake(&mut neighbor, Handshake::default())).is_ok());
    //     assert!(block_on(channels.receivers.handshake.next()).is_some());
    //
    //     assert_eq!(node.metrics.handshake_sent(), 1);
    //     assert_eq!(neighbor.metrics.handshake_sent(), 1);
    // }
    //
    // #[test]
    // fn send_milestone_request_test() {
    //     let node = Node::new();
    //     let mut channels = NeighborChannels::new();
    //     let mut neighbor = Neighbor::new(channels.senders);
    //
    //     assert_eq!(node.metrics.milestone_request_sent(), 0);
    //     assert_eq!(neighbor.metrics.milestone_request_sent(), 0);
    //
    //     assert!(channels.receivers.milestone_request.try_next().is_err());
    //     assert!(
    //         block_on(node.send_milestone_request(&mut neighbor, MilestoneRequest::default()))
    //             .is_ok()
    //     );
    //     assert!(block_on(channels.receivers.milestone_request.next()).is_some());
    //
    //     assert_eq!(node.metrics.milestone_request_sent(), 1);
    //     assert_eq!(neighbor.metrics.milestone_request_sent(), 1);
    // }
    //
    // #[test]
    // fn send_transaction_broadcast_test() {
    //     let node = Node::new();
    //     let mut channels = NeighborChannels::new();
    //     let mut neighbor = Neighbor::new(channels.senders);
    //
    //     assert_eq!(node.metrics.transaction_broadcast_sent(), 0);
    //     assert_eq!(node.metrics.transactions_sent(), 0);
    //     assert_eq!(neighbor.metrics.transaction_broadcast_sent(), 0);
    //     assert_eq!(neighbor.metrics.transactions_sent(), 0);
    //
    //     assert!(channels.receivers.transaction_broadcast.try_next().is_err());
    //     assert!(block_on(
    //         node.send_transaction_broadcast(&mut neighbor, TransactionBroadcast::default())
    //     )
    //     .is_ok());
    //     assert!(block_on(channels.receivers.transaction_broadcast.next()).is_some());
    //
    //     assert_eq!(node.metrics.transaction_broadcast_sent(), 1);
    //     assert_eq!(node.metrics.transactions_sent(), 1);
    //     assert_eq!(neighbor.metrics.transaction_broadcast_sent(), 1);
    //     assert_eq!(neighbor.metrics.transactions_sent(), 1);
    // }
    //
    // #[test]
    // fn send_transaction_request_test() {
    //     let node = Node::new();
    //     let mut channels = NeighborChannels::new();
    //     let mut neighbor = Neighbor::new(channels.senders);
    //
    //     assert_eq!(node.metrics.transaction_request_sent(), 0);
    //     assert_eq!(neighbor.metrics.transaction_request_sent(), 0);
    //
    //     assert!(channels.receivers.transaction_request.try_next().is_err());
    //     assert!(block_on(
    //         node.send_transaction_request(&mut neighbor, TransactionRequest::default())
    //     )
    //     .is_ok());
    //     assert!(block_on(channels.receivers.transaction_request.next()).is_some());
    //
    //     assert_eq!(node.metrics.transaction_request_sent(), 1);
    //     assert_eq!(neighbor.metrics.transaction_request_sent(), 1);
    // }
    //
    // #[test]
    // fn send_heartbeat_test() {
    //     let node = Node::new();
    //     let mut channels = NeighborChannels::new();
    //     let mut neighbor = Neighbor::new(channels.senders);
    //
    //     assert_eq!(node.metrics.heartbeat_sent(), 0);
    //     assert_eq!(neighbor.metrics.heartbeat_sent(), 0);
    //
    //     assert!(channels.receivers.heartbeat.try_next().is_err());
    //     assert!(block_on(node.send_heartbeat(&mut neighbor, Heartbeat::default())).is_ok());
    //     assert!(block_on(channels.receivers.heartbeat.next()).is_some());
    //
    //     assert_eq!(node.metrics.heartbeat_sent(), 1);
    //     assert_eq!(neighbor.metrics.heartbeat_sent(), 1);
    // }
}
