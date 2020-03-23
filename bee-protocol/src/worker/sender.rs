use crate::message::Message;

use bee_network::{
    EndpointId,
    Network,
};

use futures::channel::mpsc::Receiver;
use std::marker::PhantomData;

pub enum SenderWorkerEvent<M: Message> {
    Message(M),
}

pub(crate) struct SenderWorker<M: Message> {
    epid: EndpointId,
    network: Network,
    receiver: Receiver<SenderWorkerEvent<M>>,
}

impl<M: Message> SenderWorker<M> {
    pub(crate) fn new(epid: EndpointId, network: Network, receiver: Receiver<SenderWorkerEvent<M>>) -> Self {
        Self {
            epid: epid,
            network: network,
            receiver: receiver,
        }
    }

    pub(crate) async fn run(self) {}
}
