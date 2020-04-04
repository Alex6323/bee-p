use crate::{
    conf::ProtocolConf,
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
    worker::{
        BroadcasterWorker,
        BroadcasterWorkerEvent,
        MilestoneRequesterWorker,
        MilestoneRequesterWorkerEntry,
        MilestoneResponderWorker,
        MilestoneResponderWorkerEvent,
        ReceiverWorker,
        SenderContext,
        SenderWorker,
        TransactionRequesterWorker,
        TransactionRequesterWorkerEntry,
        TransactionResponderWorker,
        TransactionResponderWorkerEvent,
        TransactionWorker,
        TransactionWorkerEvent,
        WaitPriorityQueue,
    },
};

use bee_network::{
    EndpointId,
    Network,
};

use std::{
    collections::HashMap,
    ptr,
    sync::{
        atomic::AtomicU32,
        Arc,
        Mutex,
    },
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
    pub(crate) network: Network,
    pub(crate) conf: ProtocolConf,
    pub(crate) transaction_worker: (mpsc::Sender<TransactionWorkerEvent>, Mutex<Option<oneshot::Sender<()>>>),
    pub(crate) transaction_responder_worker: (
        mpsc::Sender<TransactionResponderWorkerEvent>,
        Mutex<Option<oneshot::Sender<()>>>,
    ),
    pub(crate) milestone_responder_worker: (
        mpsc::Sender<MilestoneResponderWorkerEvent>,
        Mutex<Option<oneshot::Sender<()>>>,
    ),
    pub(crate) transaction_requester_worker: WaitPriorityQueue<TransactionRequesterWorkerEntry>,
    pub(crate) milestone_requester_worker: WaitPriorityQueue<MilestoneRequesterWorkerEntry>,
    pub(crate) milestone_validator_worker: (
        mpsc::Sender<MilestoneValidatorWorkerEvent>,
        Mutex<Option<oneshot::Sender<()>>>,
    ),
    pub(crate) broadcaster_worker: (mpsc::Sender<BroadcasterWorkerEvent>, Mutex<Option<oneshot::Sender<()>>>),
    pub(crate) contexts: RwLock<HashMap<EndpointId, SenderContext>>,
    pub(crate) first_solid_milestone_index: AtomicU32,
    pub(crate) last_solid_milestone_index: AtomicU32,
}

impl Protocol {
    pub fn init(network: Network, conf: ProtocolConf) {
        if unsafe { !PROTOCOL.is_null() } {
            warn!("[Protocol ] Already initialized.");
            return;
        }

        let (transaction_worker_tx, transaction_worker_rx) = mpsc::channel(conf.transaction_worker_bound);
        let (transaction_worker_shutdown_tx, transaction_worker_shutdown_rx) = oneshot::channel();

        let (transaction_responder_worker_tx, transaction_responder_worker_rx) =
            mpsc::channel(conf.transaction_responder_worker_bound);
        let (transaction_responder_worker_shutdown_tx, transaction_responder_worker_shutdown_rx) = oneshot::channel();

        let (milestone_responder_worker_tx, milestone_responder_worker_rx) =
            mpsc::channel(conf.milestone_responder_worker_bound);
        let (milestone_responder_worker_shutdown_tx, milestone_responder_worker_shutdown_rx) = oneshot::channel();

        let (transaction_requester_worker_shutdown_tx, transaction_requester_worker_shutdown_rx) = oneshot::channel();

        let (milestone_requester_worker_shutdown_tx, milestone_requester_worker_shutdown_rx) = oneshot::channel();

        let (milestone_validator_worker_tx, milestone_validator_worker_rx) =
            mpsc::channel(conf.milestone_validator_worker_bound);
        let (milestone_validator_worker_shutdown_tx, milestone_validator_worker_shutdown_rx) = oneshot::channel();

        let (broadcaster_worker_tx, broadcaster_worker_rx) = mpsc::channel(conf.broadcaster_worker_bound);
        let (broadcaster_worker_shutdown_tx, broadcaster_worker_shutdown_rx) = oneshot::channel();

        let protocol = Protocol {
            network: network.clone(),
            conf,
            transaction_worker: (transaction_worker_tx, Mutex::new(Some(transaction_worker_shutdown_tx))),
            transaction_responder_worker: (
                transaction_responder_worker_tx,
                Mutex::new(Some(transaction_responder_worker_shutdown_tx)),
            ),
            milestone_responder_worker: (
                milestone_responder_worker_tx,
                Mutex::new(Some(milestone_responder_worker_shutdown_tx)),
            ),
            transaction_requester_worker: WaitPriorityQueue::default(),
            milestone_requester_worker: WaitPriorityQueue::default(),
            milestone_validator_worker: (
                milestone_validator_worker_tx,
                Mutex::new(Some(milestone_validator_worker_shutdown_tx)),
            ),
            broadcaster_worker: (broadcaster_worker_tx, Mutex::new(Some(broadcaster_worker_shutdown_tx))),
            contexts: RwLock::new(HashMap::new()),
            first_solid_milestone_index: AtomicU32::new(0),
            last_solid_milestone_index: AtomicU32::new(0),
        };

        unsafe {
            PROTOCOL = Box::leak(protocol.into()) as *const _;
        }

        spawn(TransactionWorker::new(transaction_worker_rx, transaction_worker_shutdown_rx).run());
        spawn(
            TransactionResponderWorker::new(
                transaction_responder_worker_rx,
                transaction_responder_worker_shutdown_rx,
            )
            .run(),
        );
        spawn(
            MilestoneResponderWorker::new(milestone_responder_worker_rx, milestone_responder_worker_shutdown_rx).run(),
        );
        spawn(TransactionRequesterWorker::new(transaction_requester_worker_shutdown_rx).run());
        spawn(MilestoneRequesterWorker::new(milestone_requester_worker_shutdown_rx).run());
        spawn(
            MilestoneValidatorWorker::new(milestone_validator_worker_rx, milestone_validator_worker_shutdown_rx).run(),
        );
        spawn(BroadcasterWorker::new(network, broadcaster_worker_rx, broadcaster_worker_shutdown_rx).run());
    }

    pub fn shutdown() {
        if let Ok(mut shutdown) = Protocol::get().transaction_worker.1.lock() {
            if let Some(shutdown) = shutdown.take() {
                if let Err(e) = shutdown.send(()) {
                    warn!("[Protocol ] Shutting down TransactionWorker failed: {:?}.", e);
                }
            }
        }
        if let Ok(mut shutdown) = Protocol::get().transaction_responder_worker.1.lock() {
            if let Some(shutdown) = shutdown.take() {
                if let Err(e) = shutdown.send(()) {
                    warn!("[Protocol ] Shutting down TransactionResponderWorker failed: {:?}.", e);
                }
            }
        }
        if let Ok(mut shutdown) = Protocol::get().milestone_responder_worker.1.lock() {
            if let Some(shutdown) = shutdown.take() {
                if let Err(e) = shutdown.send(()) {
                    warn!("[Protocol ] Shutting down MilestoneResponderWorker failed: {:?}.", e);
                }
            }
        }
        // TODO shutdown WaitPriorityQueue!
        // if let Ok(mut shutdown) = Protocol::get().transaction_requester_worker.1.lock() {
        //     if let Some(shutdown) = shutdown.take() {
        //         if let Err(e) = shutdown.send(()) {
        //             warn!("[Protocol ] Shutting down TransactionRequesterWorker failed: {:?}.", e);
        //         }
        //     }
        // }
        // if let Ok(mut shutdown) = Protocol::get().milestone_requester_worker.1.lock() {
        //     if let Some(shutdown) = shutdown.take() {
        //         if let Err(e) = shutdown.send(()) {
        //             warn!("[Protocol ] Shutting down MilestoneRequesterWorker failed: {:?}.", e);
        //         }
        //     }
        // }
        if let Ok(mut shutdown) = Protocol::get().milestone_validator_worker.1.lock() {
            if let Some(shutdown) = shutdown.take() {
                if let Err(e) = shutdown.send(()) {
                    warn!("[Protocol ] Shutting down MilestoneValidatorWorker failed: {:?}.", e);
                }
            }
        }
        if let Ok(mut shutdown) = Protocol::get().broadcaster_worker.1.lock() {
            if let Some(shutdown) = shutdown.take() {
                if let Err(e) = shutdown.send(()) {
                    warn!("[Protocol ] Shutting down BroadcasterWorker failed: {:?}.", e);
                }
            }
        }
    }

    pub(crate) fn get() -> &'static Protocol {
        if unsafe { PROTOCOL.is_null() } {
            panic!("Uninitialized protocol.");
        } else {
            unsafe { &*PROTOCOL }
        }
    }

    pub fn register(peer: Arc<Peer>, metrics: Arc<PeerMetrics>) -> (mpsc::Sender<Vec<u8>>, oneshot::Sender<()>) {
        //TODO check if not already added ?
        // ReceiverWorker
        let (receiver_tx, receiver_rx) = mpsc::channel(Protocol::get().conf.receiver_worker_bound);
        let (receiver_shutdown_tx, receiver_shutdown_rx) = oneshot::channel();

        spawn(
            ReceiverWorker::new(Protocol::get().network.clone(), peer, metrics).run(receiver_rx, receiver_shutdown_rx),
        );

        (receiver_tx, receiver_shutdown_tx)
    }

    pub(crate) async fn senders_add(network: Network, peer: Arc<Peer>, metrics: Arc<PeerMetrics>) {
        //TODO check if not already added

        // SenderWorker MilestoneRequest
        let (milestone_request_tx, milestone_request_rx) =
            mpsc::channel(Protocol::get().conf.milestone_request_send_worker_bound);
        let (milestone_request_shutdown_tx, milestone_request_shutdown_rx) = oneshot::channel();

        spawn(
            SenderWorker::<MilestoneRequest>::new(network.clone(), peer.clone(), metrics.clone())
                .run(milestone_request_rx, milestone_request_shutdown_rx),
        );

        // SenderWorker TransactionBroadcast
        let (transaction_broadcast_tx, transaction_broadcast_rx) =
            mpsc::channel(Protocol::get().conf.transaction_broadcast_send_worker_bound);
        let (transaction_broadcast_shutdown_tx, transaction_broadcast_shutdown_rx) = oneshot::channel();

        spawn(
            SenderWorker::<TransactionBroadcast>::new(network.clone(), peer.clone(), metrics.clone())
                .run(transaction_broadcast_rx, transaction_broadcast_shutdown_rx),
        );

        // SenderWorker TransactionRequest
        let (transaction_request_tx, transaction_request_rx) =
            mpsc::channel(Protocol::get().conf.transaction_request_send_worker_bound);
        let (transaction_request_shutdown_tx, transaction_request_shutdown_rx) = oneshot::channel();

        spawn(
            SenderWorker::<TransactionRequest>::new(network.clone(), peer.clone(), metrics.clone())
                .run(transaction_request_rx, transaction_request_shutdown_rx),
        );

        // SenderWorker Heartbeat
        let (heartbeat_tx, heartbeat_rx) = mpsc::channel(Protocol::get().conf.heartbeat_send_worker_bound);
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
