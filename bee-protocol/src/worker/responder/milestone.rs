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
    message::{compress_transaction_bytes, MilestoneRequest, Transaction as TransactionMessage},
    protocol::Sender,
    tangle::tangle,
};

use bee_common::{shutdown_stream::ShutdownStream, worker::Error as WorkerError};
use bee_common_ext::{node::Node, worker::Worker};
use bee_network::EndpointId;
use bee_tangle::helper::load_bundle_builder;
use bee_ternary::{T1B1Buf, T5B1Buf, TritBuf};
use bee_transaction::bundled::BundledTransaction as Transaction;

use async_trait::async_trait;
use bytemuck::cast_slice;
use futures::{
    channel::mpsc,
    stream::{Fuse, StreamExt},
};
use log::info;

pub(crate) struct MilestoneResponderWorkerEvent {
    pub(crate) epid: EndpointId,
    pub(crate) request: MilestoneRequest,
}

pub(crate) struct MilestoneResponderWorker;

#[async_trait]
impl<N: Node + 'static> Worker<N> for MilestoneResponderWorker {
    type Error = WorkerError;
    type Event = MilestoneResponderWorkerEvent;
    type Receiver = ShutdownStream<Fuse<mpsc::UnboundedReceiver<Self::Event>>>;

    async fn start(mut self, mut receiver: Self::Receiver) -> Result<(), Self::Error> {
        info!("Running.");

        while let Some(MilestoneResponderWorkerEvent { epid, request }) = receiver.next().await {
            self.process_request(epid, request).await;
        }

        info!("Stopped.");

        Ok(())
    }
}

impl MilestoneResponderWorker {
    pub(crate) fn new() -> Self {
        Self
    }

    async fn process_request(&self, epid: EndpointId, request: MilestoneRequest) {
        let index = match request.index {
            0 => tangle().get_latest_milestone_index(),
            _ => request.index.into(),
        };

        if let Some(hash) = tangle().get_milestone_hash(index) {
            if let Some(builder) = load_bundle_builder(tangle(), &hash) {
                // This is safe because the bundle has already been validated.
                let bundle = unsafe { builder.build() };
                let mut trits = TritBuf::<T1B1Buf>::zeros(Transaction::trit_len());

                for transaction in bundle {
                    transaction.into_trits_allocated(&mut trits);
                    Sender::<TransactionMessage>::send(
                        &epid,
                        TransactionMessage::new(&compress_transaction_bytes(cast_slice(
                            trits.encode::<T5B1Buf>().as_i8_slice(),
                        ))),
                    )
                    .await;
                }
            }
        }
    }
}
