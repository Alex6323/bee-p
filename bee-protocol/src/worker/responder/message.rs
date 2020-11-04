// Copyright 2020 IOTA Stiftung
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with
// the License. You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on
// an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and limitations under the License.

use crate::{
    packet::{Message as MessagePacket, MessageRequest},
    protocol::Sender,
    tangle::MsTangle,
    worker::TangleWorker,
};

use bee_common::packable::Packable;
use bee_common::{shutdown_stream::ShutdownStream, worker::Error as WorkerError};
use bee_common_ext::{node::Node, worker::Worker};
use bee_message::MessageId;
use bee_network::EndpointId;

use async_trait::async_trait;
use futures::stream::StreamExt;
use log::info;

use std::any::TypeId;

pub(crate) struct MessageResponderWorkerEvent {
    pub(crate) epid: EndpointId,
    pub(crate) request: MessageRequest,
}

pub(crate) struct MessageResponderWorker {
    pub(crate) tx: flume::Sender<MessageResponderWorkerEvent>,
}

#[async_trait]
impl<N: Node> Worker<N> for MessageResponderWorker {
    type Config = ();
    type Error = WorkerError;

    fn dependencies() -> &'static [TypeId] {
        vec![TypeId::of::<TangleWorker>()].leak()
    }

    async fn start(node: &mut N, _config: Self::Config) -> Result<Self, Self::Error> {
        let (tx, rx) = flume::unbounded();

        let tangle = node.resource::<MsTangle<N::Backend>>();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, rx.into_stream());

            while let Some(MessageResponderWorkerEvent { epid, request }) = receiver.next().await {
                if let Some(message) = tangle.get(&MessageId::from(request.message_id)).await {
                    let mut bytes = Vec::new();

                    if message.pack(&mut bytes).is_ok() {
                        Sender::<MessagePacket>::send(&epid, MessagePacket::new(&bytes));
                    }
                }
            }

            info!("Stopped.");
        });

        Ok(Self { tx })
    }
}
