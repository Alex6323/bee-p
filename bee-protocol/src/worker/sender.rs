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
    channel::mpsc::{
        Receiver,
        Sender,
    },
    stream::StreamExt,
};
use log::warn;

pub struct SenderContext {
    pub(crate) handshake_sender: Sender<SenderWorkerEvent<Handshake>>,
    pub(crate) milestone_request_sender: Sender<SenderWorkerEvent<MilestoneRequest>>,
    pub(crate) transaction_broadcast_sender: Sender<SenderWorkerEvent<TransactionBroadcast>>,
    pub(crate) transaction_request_sender: Sender<SenderWorkerEvent<TransactionRequest>>,
    pub(crate) heartbeat_sender: Sender<SenderWorkerEvent<Heartbeat>>,
}

impl SenderContext {
    pub fn new(
        handshake_sender: Sender<SenderWorkerEvent<Handshake>>,
        milestone_request_sender: Sender<SenderWorkerEvent<MilestoneRequest>>,
        transaction_broadcast_sender: Sender<SenderWorkerEvent<TransactionBroadcast>>,
        transaction_request_sender: Sender<SenderWorkerEvent<TransactionRequest>>,
        heartbeat_sender: Sender<SenderWorkerEvent<Heartbeat>>,
    ) -> Self {
        Self {
            handshake_sender,
            milestone_request_sender,
            transaction_broadcast_sender,
            transaction_request_sender,
            heartbeat_sender,
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

    pub fn contexts(&self) -> &RwLock<HashMap<EndpointId, SenderContext>> {
        &self.contexts
    }
}

pub static mut SENDER_REGISTRY: *const SenderRegistry = ptr::null();

pub fn sender_registry() -> &'static SenderRegistry {
    if unsafe { SENDER_REGISTRY.is_null() } {
        panic!("Uninitialized sender registry.");
    } else {
        unsafe { &*SENDER_REGISTRY }
    }
}

pub enum SenderWorkerEvent<M: Message> {
    Message(M),
}

pub struct SenderWorker<M: Message> {
    peer: Arc<Peer>,
    metrics: Arc<PeerMetrics>,
    network: Network,
    receiver: Receiver<SenderWorkerEvent<M>>,
}

macro_rules! implement_sender_worker {
    ($type:ident, $incrementor:ident) => {
        impl SenderWorker<$type> {
            pub fn new(
                peer: Arc<Peer>,
                metrics: Arc<PeerMetrics>,
                network: Network,
                receiver: Receiver<SenderWorkerEvent<$type>>,
            ) -> Self {
                Self {
                    peer,
                    metrics,
                    network,
                    receiver,
                }
            }

            pub async fn run(mut self) {
                while let Some(SenderWorkerEvent::Message(message)) = self.receiver.next().await {
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
        }
    };
}

implement_sender_worker!(Handshake, handshake_sent);
implement_sender_worker!(MilestoneRequest, milestone_request_sent);
implement_sender_worker!(TransactionBroadcast, transaction_broadcast_sent);
implement_sender_worker!(TransactionRequest, transaction_request_sent);
implement_sender_worker!(Heartbeat, heartbeat_sent);
