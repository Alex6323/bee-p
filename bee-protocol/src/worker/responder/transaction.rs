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
    tangle::tangle,
    worker::SenderWorker,
};

use bee_crypto::ternary::Hash;
use bee_network::EndpointId;
use bee_ternary::{T1B1Buf, T5B1Buf, TritBuf, Trits, T5B1};
use bee_transaction::bundled::{BundledTransaction as Transaction, BundledTransactionField};

use bytemuck::cast_slice;
use futures::{
    channel::{mpsc, oneshot},
    future::FutureExt,
    select,
    stream::StreamExt,
};
use log::info;

pub(crate) struct TransactionResponderWorkerEvent {
    pub(crate) epid: EndpointId,
    pub(crate) request: TransactionRequest,
}

pub(crate) struct TransactionResponderWorker {}

impl TransactionResponderWorker {
    pub(crate) fn new() -> Self {
        Self {}
    }

    async fn process_request(&self, epid: EndpointId, request: TransactionRequest) {
        match Trits::<T5B1>::try_from_raw(cast_slice(&request.hash), Hash::trit_len()) {
            Ok(hash) => {
                match tangle().get(&Hash::from_inner_unchecked(hash.encode())) {
                    Some(transaction) => {
                        let mut trits = TritBuf::<T1B1Buf>::zeros(Transaction::trit_len());
                        transaction.into_trits_allocated(&mut trits);
                        // TODO dedicated channel ? Priority Queue ?
                        SenderWorker::<TransactionMessage>::send(
                            &epid,
                            // TODO try to compress lower in the pipeline ?
                            TransactionMessage::new(&compress_transaction_bytes(cast_slice(
                                trits.encode::<T5B1Buf>().as_i8_slice(),
                            ))),
                        )
                        .await;
                    }
                    None => {}
                }
            }
            Err(_) => {}
        }
    }

    pub(crate) async fn run(
        self,
        receiver: mpsc::Receiver<TransactionResponderWorkerEvent>,
        shutdown: oneshot::Receiver<()>,
    ) {
        info!("Running.");

        let mut receiver_fused = receiver.fuse();
        let mut shutdown_fused = shutdown.fuse();

        loop {
            select! {
                event = receiver_fused.next() => {
                    if let Some(TransactionResponderWorkerEvent { epid, request }) = event {
                        self.process_request(epid, request).await;
                    }
                },
                _ = shutdown_fused => {
                    break;
                }
            }
        }

        info!("Stopped.");
    }
}
