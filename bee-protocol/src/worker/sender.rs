use crate::message::{
    Handshake,
    Heartbeat,
    Message,
    MilestoneRequest,
    TransactionBroadcast,
    TransactionRequest,
};

use bee_network::Command::SendBytes;
use bee_network::{
    EndpointId,
    Network,
};

use async_std::sync::RwLock;
use futures::channel::mpsc::{
    Receiver,
    Sender,
};
use futures::stream::StreamExt;

use std::collections::HashMap;
use std::marker::PhantomData;
use std::ptr;

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
    epid: EndpointId,
    network: Network,
    receiver: Receiver<SenderWorkerEvent<M>>,
}

impl<M: Message> SenderWorker<M> {
    pub fn new(epid: EndpointId, network: Network, receiver: Receiver<SenderWorkerEvent<M>>) -> Self {
        Self {
            epid: epid,
            network: network,
            receiver: receiver,
        }
    }

    pub async fn run(mut self) {
        // TODO metric ?
        while let Some(SenderWorkerEvent::Message(message)) = self.receiver.next().await {
            self.network
                .send(SendBytes {
                    epid: self.epid,
                    bytes: message.into_full_bytes(),
                    responder: None,
                })
                .await;
        }
    }
}
