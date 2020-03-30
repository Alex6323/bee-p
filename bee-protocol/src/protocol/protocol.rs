use crate::{
    message::{
        Heartbeat,
        MilestoneRequest,
        TransactionBroadcast,
        TransactionRequest,
    },
    milestone::{
        MilestoneIndex,
        MilestoneValidatorWorker,
        MilestoneValidatorWorkerEvent,
    },
    peer::{
        Peer,
        PeerMetrics,
    },
    protocol::{
        HEARTBEAT_SEND_BOUND,
        MILESTONE_REQUEST_SEND_BOUND,
        TRANSACTION_BROADCAST_SEND_BOUND,
        TRANSACTION_REQUEST_SEND_BOUND,
    },
    worker::{
        MilestoneRequesterWorker,
        MilestoneRequesterWorkerEvent,
        MilestoneResponderWorker,
        MilestoneResponderWorkerEvent,
        ReceiverWorker,
        SenderContext,
        SenderWorker,
        TransactionRequesterWorker,
        TransactionRequesterWorkerEvent,
        TransactionResponderWorker,
        TransactionResponderWorkerEvent,
        TransactionWorker,
        TransactionWorkerEvent,
    },
};

use bee_network::{
    EndpointId,
    Network,
};

use std::{
    collections::HashMap,
    ptr,
    sync::Arc,
};

use async_std::{
    sync::RwLock,
    task::spawn,
};
use futures::channel::{
    mpsc,
    oneshot,
};
use log::warn;

static mut PROTOCOL: *const Protocol = ptr::null();

pub struct Protocol {
    pub(crate) transaction_worker: mpsc::Sender<TransactionWorkerEvent>,
    pub(crate) transaction_responder_worker: mpsc::Sender<TransactionResponderWorkerEvent>,
    pub(crate) milestone_responder_worker: mpsc::Sender<MilestoneResponderWorkerEvent>,
    pub(crate) transaction_requester_worker: mpsc::Sender<TransactionRequesterWorkerEvent>,
    pub(crate) milestone_requester_worker: mpsc::Sender<MilestoneRequesterWorkerEvent>,
    pub(crate) milestone_validator_worker: mpsc::Sender<MilestoneValidatorWorkerEvent>,
    pub(crate) contexts: RwLock<HashMap<EndpointId, SenderContext>>,
}

impl Protocol {
    pub fn init() {
        if unsafe { !PROTOCOL.is_null() } {
            warn!("[Protocol ] Already initialized.");
            return;
        }

        // TODO conf
        let (milestone_validator_worker_sender, milestone_validator_worker_receiver) = mpsc::channel(1000);
        // TODO conf
        let (transaction_worker_sender, transaction_worker_receiver) = mpsc::channel(1000);
        // TODO conf
        let (transaction_responder_worker_sender, transaction_responder_worker_receiver) = mpsc::channel(1000);
        // TODO conf
        let (milestone_responder_worker_sender, milestone_responder_worker_receiver) = mpsc::channel(1000);
        // TODO conf
        let (transaction_requester_worker_sender, transaction_requester_worker_receiver) = mpsc::channel(1000);
        // TODO conf
        let (milestone_requester_worker_sender, milestone_requester_worker_receiver) = mpsc::channel(1000);

        let protocol = Protocol {
            transaction_worker: transaction_worker_sender,
            transaction_responder_worker: transaction_responder_worker_sender,
            milestone_responder_worker: milestone_responder_worker_sender,
            transaction_requester_worker: transaction_requester_worker_sender,
            milestone_requester_worker: milestone_requester_worker_sender,
            milestone_validator_worker: milestone_validator_worker_sender,
            contexts: RwLock::new(HashMap::new()),
        };

        unsafe {
            PROTOCOL = Box::leak(protocol.into()) as *const _;
        }

        spawn(MilestoneValidatorWorker::new(milestone_validator_worker_receiver).run());
        spawn(TransactionWorker::new(transaction_worker_receiver).run());
        spawn(TransactionResponderWorker::new(transaction_responder_worker_receiver).run());
        spawn(MilestoneResponderWorker::new(milestone_responder_worker_receiver).run());
        spawn(TransactionRequesterWorker::new(transaction_requester_worker_receiver).run());
        spawn(MilestoneRequesterWorker::new(milestone_requester_worker_receiver).run());
    }

    pub fn shutdown() {
        //shutdown main workers
    }

    pub(crate) fn get() -> &'static Protocol {
        if unsafe { PROTOCOL.is_null() } {
            panic!("Uninitialized protocol.");
        } else {
            unsafe { &*PROTOCOL }
        }
    }

    pub fn register(
        network: Network,
        peer: Arc<Peer>,
        metrics: Arc<PeerMetrics>,
    ) -> (mpsc::Sender<Vec<u8>>, oneshot::Sender<()>) {
        //TODO check if not already added ?
        // TODO conf
        // ReceiverWorker
        let (receiver_tx, receiver_rx) = mpsc::channel(1000);
        let (receiver_shutdown_tx, receiver_shutdown_rx) = oneshot::channel();

        spawn(ReceiverWorker::new(network, peer, metrics).run(receiver_rx, receiver_shutdown_rx));

        (receiver_tx, receiver_shutdown_tx)
    }

    pub(crate) async fn senders_add(network: Network, peer: Arc<Peer>, metrics: Arc<PeerMetrics>) {
        //TODO check if not already added

        // SenderWorker MilestoneRequest
        let (milestone_request_tx, milestone_request_rx) = mpsc::channel(MILESTONE_REQUEST_SEND_BOUND);
        let (milestone_request_shutdown_tx, milestone_request_shutdown_rx) = oneshot::channel();

        spawn(
            SenderWorker::<MilestoneRequest>::new(network.clone(), peer.clone(), metrics.clone())
                .run(milestone_request_rx, milestone_request_shutdown_rx),
        );

        // SenderWorker TransactionBroadcast
        let (transaction_broadcast_tx, transaction_broadcast_rx) = mpsc::channel(TRANSACTION_BROADCAST_SEND_BOUND);
        let (transaction_broadcast_shutdown_tx, transaction_broadcast_shutdown_rx) = oneshot::channel();

        spawn(
            SenderWorker::<TransactionBroadcast>::new(network.clone(), peer.clone(), metrics.clone())
                .run(transaction_broadcast_rx, transaction_broadcast_shutdown_rx),
        );

        // SenderWorker TransactionRequest
        let (transaction_request_tx, transaction_request_rx) = mpsc::channel(TRANSACTION_REQUEST_SEND_BOUND);
        let (transaction_request_shutdown_tx, transaction_request_shutdown_rx) = oneshot::channel();

        spawn(
            SenderWorker::<TransactionRequest>::new(network.clone(), peer.clone(), metrics.clone())
                .run(transaction_request_rx, transaction_request_shutdown_rx),
        );

        // SenderWorker Heartbeat
        let (heartbeat_tx, heartbeat_rx) = mpsc::channel(HEARTBEAT_SEND_BOUND);
        let (heartbeat_shutdown_tx, heartbeat_shutdown_rx) = oneshot::channel();

        spawn(
            SenderWorker::<Heartbeat>::new(network.clone(), peer.clone(), metrics.clone())
                .run(heartbeat_rx, heartbeat_shutdown_rx),
        );

        let context = SenderContext::new(
            (milestone_request_tx, milestone_request_shutdown_tx),
            (transaction_broadcast_tx, transaction_broadcast_shutdown_tx),
            (transaction_request_tx, transaction_request_shutdown_tx),
            (heartbeat_tx, heartbeat_shutdown_tx),
        );

        Protocol::get().contexts.write().await.insert(peer.epid, context);
    }

    pub(crate) async fn senders_remove(epid: &EndpointId) {
        if let Some(context) = Protocol::get().contexts.write().await.remove(epid) {
            if let Err(_) = context.milestone_request.1.send(()) {
                warn!("[Protocol ] Shutting down MilestoneRequest SenderWorker failed.");
            }
            if let Err(_) = context.transaction_broadcast.1.send(()) {
                warn!("[Protocol ] Shutting down TransactionBroadcast SenderWorker failed.");
            }
            if let Err(_) = context.transaction_request.1.send(()) {
                warn!("[Protocol ] Shutting down TransactionRequest SenderWorker failed.");
            }
            if let Err(_) = context.heartbeat.1.send(()) {
                warn!("[Protocol ] Shutting down Heartbeat SenderWorker failed.");
            }
        }
    }

    // Helpers

    pub async fn send_heartbeat(
        epid: EndpointId,
        first_solid_milestone_index: MilestoneIndex,
        last_solid_milestone_index: MilestoneIndex,
    ) {
        SenderWorker::<Heartbeat>::send(
            &epid,
            Heartbeat::new(first_solid_milestone_index, last_solid_milestone_index),
        )
        .await;
    }

    pub async fn broadcast_heartbeat(
        first_solid_milestone_index: MilestoneIndex,
        last_solid_milestone_index: MilestoneIndex,
    ) {
        SenderWorker::<Heartbeat>::broadcast(Heartbeat::new(first_solid_milestone_index, last_solid_milestone_index))
            .await;
    }

    pub async fn send_milestone_request(epid: EndpointId, index: MilestoneIndex) {
        SenderWorker::<MilestoneRequest>::send(&epid, MilestoneRequest::new(index)).await;
    }

    pub async fn broadcast_milestone_request(index: MilestoneIndex) {
        SenderWorker::<MilestoneRequest>::broadcast(MilestoneRequest::new(index)).await;
    }

    pub async fn send_transaction(epid: EndpointId, transaction: &[u8]) {
        SenderWorker::<TransactionBroadcast>::send(&epid, TransactionBroadcast::new(transaction)).await;
    }

    pub async fn broadcast_transaction(transaction: &[u8]) {
        SenderWorker::<TransactionBroadcast>::broadcast(TransactionBroadcast::new(transaction)).await;
    }

    //  TODO constant

    pub async fn send_transaction_request(epid: EndpointId, hash: [u8; 49]) {
        SenderWorker::<TransactionRequest>::send(&epid, TransactionRequest::new(hash)).await;
    }

    pub async fn broadcast_transaction_request(hash: [u8; 49]) {
        SenderWorker::<TransactionRequest>::broadcast(TransactionRequest::new(hash)).await;
    }
}
