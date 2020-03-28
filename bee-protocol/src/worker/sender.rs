use crate::{
    message::{
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
    marker::PhantomData,
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

pub(crate) struct SenderContext {
    pub(crate) milestone_request: (mpsc::Sender<SenderWorkerEvent<MilestoneRequest>>, oneshot::Sender<()>),
    pub(crate) transaction_broadcast: (
        mpsc::Sender<SenderWorkerEvent<TransactionBroadcast>>,
        oneshot::Sender<()>,
    ),
    pub(crate) transaction_request: (mpsc::Sender<SenderWorkerEvent<TransactionRequest>>, oneshot::Sender<()>),
    pub(crate) heartbeat: (mpsc::Sender<SenderWorkerEvent<Heartbeat>>, oneshot::Sender<()>),
}

impl SenderContext {
    pub(crate) fn new(
        milestone_request: (mpsc::Sender<SenderWorkerEvent<MilestoneRequest>>, oneshot::Sender<()>),
        transaction_broadcast: (
            mpsc::Sender<SenderWorkerEvent<TransactionBroadcast>>,
            oneshot::Sender<()>,
        ),
        transaction_request: (mpsc::Sender<SenderWorkerEvent<TransactionRequest>>, oneshot::Sender<()>),
        heartbeat: (mpsc::Sender<SenderWorkerEvent<Heartbeat>>, oneshot::Sender<()>),
    ) -> Self {
        Self {
            milestone_request,
            transaction_broadcast,
            transaction_request,
            heartbeat,
        }
    }

    pub(crate) fn shutdown(self) {
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
pub(crate) struct SenderRegistry {
    contexts: RwLock<HashMap<EndpointId, SenderContext>>,
}

impl SenderRegistry {
    pub(crate) fn init() {
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

    pub(crate) async fn insert(epid: EndpointId, context: SenderContext) {
        SenderRegistry::registry().contexts.write().await.insert(epid, context);
    }

    pub(crate) async fn remove(epid: &EndpointId) -> Option<SenderContext> {
        SenderRegistry::registry().contexts.write().await.remove(epid)
    }
}

static mut SENDER_REGISTRY: *const SenderRegistry = ptr::null();

pub(crate) enum SenderWorkerEvent<M: Message> {
    Message(M),
}

pub(crate) struct SenderWorker<M: Message> {
    peer: Arc<Peer>,
    metrics: Arc<PeerMetrics>,
    network: Network,
    _message_type: PhantomData<M>,
}

macro_rules! implement_sender_worker {
    ($type:ty, $sender:tt, $incrementor:tt) => {
        impl SenderWorker<$type> {
            pub(crate) fn new(peer: Arc<Peer>, metrics: Arc<PeerMetrics>, network: Network) -> Self {
                Self {
                    peer,
                    metrics,
                    network,
                    _message_type: PhantomData,
                }
            }

            pub(crate) async fn send(epid: &EndpointId, message: $type) {
                if let Some(context) = SenderRegistry::registry().contexts.read().await.get(&epid) {
                    if let Err(e) = context
                        .$sender
                        .0
                        // TODO avoid clone ?
                        .clone()
                        .send(SenderWorkerEvent::Message(message))
                        .await
                    {
                        warn!("[SenderWorker ] Sending message failed: {:?}.", e);
                    }
                };
            }

            pub(crate) async fn broadcast(message: $type) {
                for context in SenderRegistry::registry().contexts.read().await.values() {
                    if let Err(e) = context
                        .$sender
                        .0
                        // TODO avoid clone ?
                        .clone()
                        .send(SenderWorkerEvent::Message(message.clone()))
                        .await
                    {
                        warn!("[SenderWorker ] Sending message failed: {:?}.", e);
                    }
                }
            }

            pub(crate) async fn run(
                mut self,
                events_receiver: mpsc::Receiver<SenderWorkerEvent<$type>>,
                shutdown_receiver: oneshot::Receiver<()>,
            ) {
                let mut events = events_receiver.fuse();
                let mut shutdown = shutdown_receiver.fuse();

                loop {
                    select! {
                        message = events.next() => {
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
                        _ = shutdown => {
                            break;
                        }
                    }
                }
            }
        }
    };
}

implement_sender_worker!(MilestoneRequest, milestone_request, milestone_request_sent);
implement_sender_worker!(TransactionBroadcast, transaction_broadcast, transaction_broadcast_sent);
implement_sender_worker!(TransactionRequest, transaction_request, transaction_request_sent);
implement_sender_worker!(Heartbeat, heartbeat, heartbeat_sent);
