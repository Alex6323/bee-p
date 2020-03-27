use crate::{
    message::{
        Handshake,
        Heartbeat,
        Message,
        MilestoneRequest,
        TransactionBroadcast,
        TransactionRequest,
    },
    peer::{
        Peer,
        PeerMetrics,
    },
};

use bee_network::{
    Command::SendBytes,
    EndpointId,
    Network,
};

use std::{
    collections::HashMap,
    ptr,
    sync::Arc,
};

use async_std::sync::RwLock;
use futures::{
    channel::{
        mpsc,
        oneshot,
    },
    future::FutureExt,
    select,
    sink::SinkExt,
    stream::StreamExt,
};
use log::warn;

pub struct SenderContext {
    pub(crate) handshake: (mpsc::Sender<SenderWorkerEvent<Handshake>>, oneshot::Sender<()>),
    pub(crate) milestone_request: (mpsc::Sender<SenderWorkerEvent<MilestoneRequest>>, oneshot::Sender<()>),
    pub(crate) transaction_broadcast: (
        mpsc::Sender<SenderWorkerEvent<TransactionBroadcast>>,
        oneshot::Sender<()>,
    ),
    pub(crate) transaction_request: (mpsc::Sender<SenderWorkerEvent<TransactionRequest>>, oneshot::Sender<()>),
    pub(crate) heartbeat: (mpsc::Sender<SenderWorkerEvent<Heartbeat>>, oneshot::Sender<()>),
}

impl SenderContext {
    pub fn new(
        handshake: (mpsc::Sender<SenderWorkerEvent<Handshake>>, oneshot::Sender<()>),
        milestone_request: (mpsc::Sender<SenderWorkerEvent<MilestoneRequest>>, oneshot::Sender<()>),
        transaction_broadcast: (
            mpsc::Sender<SenderWorkerEvent<TransactionBroadcast>>,
            oneshot::Sender<()>,
        ),
        transaction_request: (mpsc::Sender<SenderWorkerEvent<TransactionRequest>>, oneshot::Sender<()>),
        heartbeat: (mpsc::Sender<SenderWorkerEvent<Heartbeat>>, oneshot::Sender<()>),
    ) -> Self {
        Self {
            handshake,
            milestone_request,
            transaction_broadcast,
            transaction_request,
            heartbeat,
        }
    }

    pub fn shutdown(self) {
        if let Err(_) = self.handshake.1.send(()) {
            warn!("[SenderContext ] Shutting down Handshake SenderWorker failed.");
        }
        if let Err(_) = self.milestone_request.1.send(()) {
            warn!("[SenderContext ] Shutting down MilestoneRequest SenderWorker failed.");
        }
        if let Err(_) = self.transaction_broadcast.1.send(()) {
            warn!("[SenderContext ] Shutting down TransactionBroadcast SenderWorker failed.");
        }
        if let Err(_) = self.transaction_request.1.send(()) {
            warn!("[SenderContext ] Shutting down TransactionRequest SenderWorker failed.");
        }
        if let Err(_) = self.heartbeat.1.send(()) {
            warn!("[SenderContext ] Shutting down Heartbeat SenderWorker failed.");
        }
    }
}

#[derive(Default)]
pub struct SenderRegistry {
    contexts: RwLock<HashMap<EndpointId, SenderContext>>,
}

impl SenderRegistry {
    pub fn init() {
        unsafe {
            SENDER_REGISTRY = Box::leak(SenderRegistry::default().into()) as *const _;
        }
    }

    fn registry() -> &'static SenderRegistry {
        if unsafe { SENDER_REGISTRY.is_null() } {
            panic!("Uninitialized sender registry.");
        } else {
            unsafe { &*SENDER_REGISTRY }
        }
    }

    pub async fn insert(epid: EndpointId, context: SenderContext) {
        SenderRegistry::registry().contexts.write().await.insert(epid, context);
    }

    pub async fn remove(epid: &EndpointId) -> Option<SenderContext> {
        SenderRegistry::registry().contexts.write().await.remove(epid)
    }
}

static mut SENDER_REGISTRY: *const SenderRegistry = ptr::null();

pub enum SenderWorkerEvent<M: Message> {
    Message(M),
}

pub struct SenderWorker<M: Message> {
    peer: Arc<Peer>,
    metrics: Arc<PeerMetrics>,
    network: Network,
    events: mpsc::Receiver<SenderWorkerEvent<M>>,
    shutdown: oneshot::Receiver<()>,
}

macro_rules! implement_sender_worker {
    ($type:ty, $sender:tt, $incrementor:tt) => {
        impl SenderWorker<$type> {
            pub fn new(
                peer: Arc<Peer>,
                metrics: Arc<PeerMetrics>,
                network: Network,
                events: mpsc::Receiver<SenderWorkerEvent<$type>>,
                shutdown: oneshot::Receiver<()>,
            ) -> Self {
                Self {
                    peer,
                    metrics,
                    network,
                    events,
                    shutdown,
                }
            }

            pub async fn send(epid: &EndpointId, message: $type) {
                if let Some(context) = SenderRegistry::registry().contexts.read().await.get(&epid) {
                    if let Err(e) = context
                        .$sender
                        .0
                        // TODO avoid clone
                        .clone()
                        .send(SenderWorkerEvent::Message(message))
                        .await
                    {
                        warn!("[SenderWorker ] Sending message failed: {:?}.", e);
                    }
                };
            }

            pub async fn broadcast(message: $type) {
                for key in SenderRegistry::registry().contexts.read().await.keys() {
                    SenderWorker::<$type>::send(key, message.clone()).await;
                }
            }

            pub async fn run(mut self) {
                loop {
                    select! {
                        message = self.events.next().fuse() => {
                            if let Some(SenderWorkerEvent::Message(message)) = message {
                                match self
                                    .network
                                    .send(SendBytes {
                                        epid: self.peer.epid,
                                        bytes: message.into_full_bytes(),
                                        responder: None,
                                    })
                                    .await
                                {
                                    Ok(_) => {
                                        self.peer.metrics.$incrementor();
                                        self.metrics.$incrementor();
                                    }
                                    Err(e) => {
                                        warn!(
                                            "[SenderWorker({}) ] Sending message failed: {}.",
                                            self.peer.epid, e
                                        );
                                    }
                                }
                            }
                        }
                        _ = (&mut self.shutdown).fuse() => {
                            break;
                        }
                    }
                }
            }
        }
    };
}

implement_sender_worker!(Handshake, handshake, handshake_sent);
implement_sender_worker!(MilestoneRequest, milestone_request, milestone_request_sent);
implement_sender_worker!(TransactionBroadcast, transaction_broadcast, transaction_broadcast_sent);
implement_sender_worker!(TransactionRequest, transaction_request, transaction_request_sent);
implement_sender_worker!(Heartbeat, heartbeat, heartbeat_sent);
