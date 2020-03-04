use crate::message::Message;
use netzwerk::{Network, PeerId};

use std::marker::PhantomData;

pub(crate) struct NeighborSenderActor<M> {
    peer_id: PeerId,
    network: Network,
    message_type: PhantomData<M>,
}

impl<M: Message> NeighborSenderActor<M> {
    pub(crate) fn new(peer_id: PeerId, network: Network) -> Self {
        Self {
            peer_id: peer_id,
            network: network,
            message_type: PhantomData,
        }
    }

    pub(crate) async fn run(self) {}
}
