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
    tangle::tangle,
    worker::SenderWorker,
};

use bee_common::{shutdown_stream::ShutdownStream, worker::Error as WorkerError};
use bee_network::EndpointId;
use bee_ternary::{T1B1Buf, T5B1Buf, TritBuf};
use bee_transaction::bundled::BundledTransaction as Transaction;

use bytemuck::cast_slice;
use futures::{channel::mpsc, stream::StreamExt};

use log::info;

type Receiver = ShutdownStream<mpsc::Receiver<MilestoneResponderWorkerEvent>>;

pub(crate) struct MilestoneResponderWorkerEvent {
    pub(crate) epid: EndpointId,
    pub(crate) request: MilestoneRequest,
}

pub(crate) struct MilestoneResponderWorker {
    receiver: Receiver,
}

impl MilestoneResponderWorker {
    pub(crate) fn new(receiver: Receiver) -> Self {
        Self { receiver }
    }

    fn process_request(&self, epid: EndpointId, request: MilestoneRequest) {
        let index = match request.index {
            0 => tangle().get_last_milestone_index(),
            _ => request.index.into(),
        };

        // TODO send complete ms bundle ?
        match tangle().get_milestone(index) {
            Some(transaction) => {
                let mut trits = TritBuf::<T1B1Buf>::zeros(Transaction::trit_len());
                transaction.into_trits_allocated(&mut trits);
                // TODO dedicated channel ? Priority Queue ?
                // TODO compress bytes
                SenderWorker::<TransactionMessage>::send(
                    &epid,
                    // TODO try to compress lower in the pipeline ?
                    TransactionMessage::new(&compress_transaction_bytes(cast_slice(
                        trits.encode::<T5B1Buf>().as_i8_slice(),
                    ))),
                );
            }
            None => return,
        }
    }

    pub(crate) async fn run(mut self) -> Result<(), WorkerError> {
        info!("Running.");

        while let Some(MilestoneResponderWorkerEvent { epid, request }) = self.receiver.next().await {
            self.process_request(epid, request);
        }

        info!("Stopped.");

        Ok(())
    }
}
