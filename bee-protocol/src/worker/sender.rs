use crate::message::Message;

use bee_network::{EndpointId, Network};

use std::marker::PhantomData;

pub(crate) struct SenderWorker<M> {
    epid: EndpointId,
    network: Network,
    message_type: PhantomData<M>,
}

impl<M: Message> SenderWorker<M> {
    pub(crate) fn new(epid: EndpointId, network: Network) -> Self {
        Self {
            epid: epid,
            network: network,
            message_type: PhantomData,
        }
    }

    pub(crate) async fn run(self) {}
}
