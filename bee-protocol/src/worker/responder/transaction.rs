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
    message::{compress_transaction_bytes, Transaction as TransactionMessage, TransactionRequest},
    protocol::Sender,
    tangle::tangle,
};

use bee_common::{shutdown_stream::ShutdownStream, worker::Error as WorkerError};
use bee_common_ext::{node::Node, worker::Worker};
use bee_crypto::ternary::Hash;
use bee_network::EndpointId;
use bee_ternary::{T1B1Buf, T5B1Buf, TritBuf, Trits, T5B1};
use bee_transaction::bundled::{BundledTransaction as Transaction, BundledTransactionField};

use async_trait::async_trait;
use bytemuck::cast_slice;
use futures::{
    channel::mpsc,
    stream::{Fuse, StreamExt},
};
use log::info;

pub(crate) struct TransactionResponderWorkerEvent {
    pub(crate) epid: EndpointId,
    pub(crate) request: TransactionRequest,
}

pub(crate) struct TransactionResponderWorker;

#[async_trait]
impl<N: Node + 'static> Worker<N> for TransactionResponderWorker {
    type Event = TransactionResponderWorkerEvent;
    type Receiver = ShutdownStream<Fuse<mpsc::UnboundedReceiver<Self::Event>>>;

    async fn run(mut self, mut receiver: Self::Receiver) -> Result<(), WorkerError> {
        info!("Running.");

        while let Some(TransactionResponderWorkerEvent { epid, request }) = receiver.next().await {
            self.process_request(epid, request).await;
        }

        info!("Stopped.");

        Ok(())
    }
}

impl TransactionResponderWorker {
    pub(crate) fn new() -> Self {
        Self
    }

    async fn process_request(&self, epid: EndpointId, request: TransactionRequest) {
        if let Ok(hash) = Trits::<T5B1>::try_from_raw(cast_slice(&request.hash), Hash::trit_len()) {
            if let Some(transaction) = tangle().get(&Hash::from_inner_unchecked(hash.encode())) {
                let mut trits = TritBuf::<T1B1Buf>::zeros(Transaction::trit_len());

                transaction.into_trits_allocated(&mut trits);
                Sender::<TransactionMessage>::send(
                    &epid,
                    TransactionMessage::new(&compress_transaction_bytes(cast_slice(
                        trits.encode::<T5B1Buf>().as_i8_slice(),
                    ))),
                )
                .await
            }
        }
    }
}
