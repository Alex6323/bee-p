use crate::{
    message::{
        Heartbeat,
        MilestoneRequest,
        TransactionBroadcast,
        TransactionRequest,
    },
    milestone::{
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
        ReceiverWorker,
        RequesterWorker,
        RequesterWorkerEvent,
        ResponderWorker,
        ResponderWorkerEvent,
        SenderContext,
        SenderWorker,
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
    pub(crate) responder_worker: mpsc::Sender<ResponderWorkerEvent>,
    pub(crate) requester_worker: mpsc::Sender<RequesterWorkerEvent>,
    pub(crate) milestone_validator_worker: mpsc::Sender<MilestoneValidatorWorkerEvent>,
    pub(crate) contexts: RwLock<HashMap<EndpointId, SenderContext>>,
}

impl Protocol {
    pub fn init() {
        // TODO only init once

        // TODO conf
        let (milestone_validator_worker_sender, milestone_validator_worker_receiver) = mpsc::channel(1000);
        spawn(MilestoneValidatorWorker::new(milestone_validator_worker_receiver).run());

        // TODO conf
        let (transaction_worker_sender, transaction_worker_receiver) = mpsc::channel(1000);
        spawn(TransactionWorker::new(transaction_worker_receiver).run());

        // TODO conf
        let (responder_worker_sender, responder_worker_receiver) = mpsc::channel(1000);
        spawn(ResponderWorker::new(responder_worker_receiver).run());

        // TODO conf
        let (requester_worker_sender, requester_worker_receiver) = mpsc::channel(1000);
        spawn(RequesterWorker::new(requester_worker_receiver).run());

        let protocol = Protocol {
            transaction_worker: transaction_worker_sender,
            responder_worker: responder_worker_sender,
            requester_worker: requester_worker_sender,
            milestone_validator_worker: milestone_validator_worker_sender,
            contexts: RwLock::new(HashMap::new()),
        };

        unsafe {
            PROTOCOL = Box::leak(protocol.into()) as *const _;
        }
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
}
