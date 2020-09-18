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
    worker::Worker,
};

use bee_common::{shutdown_stream::ShutdownStream, worker::Error as WorkerError};
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
impl Worker for TransactionResponderWorker {
    type Event = TransactionResponderWorkerEvent;
    type Receiver = ShutdownStream<Fuse<mpsc::UnboundedReceiver<Self::Event>>>;

    fn name() -> &'static str {
        "transaction_responder_worker"
    }

    fn dependencies() -> &'static [&'static str] {
        &[]
    }

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
        match Trits::<T5B1>::try_from_raw(cast_slice(&request.hash), Hash::trit_len()) {
            Ok(hash) => {
                match tangle().get(&Hash::from_inner_unchecked(hash.encode())) {
                    Some(transaction) => {
                        let mut trits = TritBuf::<T1B1Buf>::zeros(Transaction::trit_len());
                        transaction.into_trits_allocated(&mut trits);
                        // TODO dedicated channel ? Priority Queue ?
                        Sender::<TransactionMessage>::send(
                            &epid,
                            // TODO try to compress lower in the pipeline ?
                            TransactionMessage::new(&compress_transaction_bytes(cast_slice(
                                trits.encode::<T5B1Buf>().as_i8_slice(),
                            ))),
                        )
                        .await
                    }
                    None => {}
                }
            }
            Err(_) => {}
        }
    }
}
