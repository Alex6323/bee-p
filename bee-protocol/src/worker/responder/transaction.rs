use crate::{
    message::{
        TransactionBroadcast,
        TransactionRequest,
    },
    util::compress_transaction_bytes,
    worker::SenderWorker,
};

use bee_bundle::{
    Hash,
    Transaction,
    TransactionField,
};
use bee_network::EndpointId;
use bee_tangle::tangle;
use bee_ternary::{
    T1B1Buf,
    T5B1Buf,
    TritBuf,
    Trits,
    T5B1,
};

use bytemuck::cast_slice;
use futures::{
    channel::{
        mpsc,
        oneshot,
    },
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
                match tangle()
                    .get_transaction(&Hash::from_inner_unchecked(hash.to_buf()))
                    .await
                {
                    Some(transaction) => {
                        let mut trits = TritBuf::<T1B1Buf>::zeros(Transaction::trits_len());
                        transaction.into_trits_allocated(&mut trits);
                        // TODO dedicated channel ? Priority Queue ?
                        SenderWorker::<TransactionBroadcast>::send(
                            &epid,
                            // TODO try to compress lower in the pipeline ?
                            TransactionBroadcast::new(&compress_transaction_bytes(cast_slice(
                                trits.encode::<T5B1Buf>().as_i8_slice(),
                            ))),
                        )
                        .await;
                    }
                    None => return,
                }
            }
            Err(_) => return,
        }
    }

    pub(crate) async fn run(
        self,
        receiver: mpsc::Receiver<TransactionResponderWorkerEvent>,
        shutdown: oneshot::Receiver<()>,
    ) {
        info!("[TransactionResponderWorker ] Running.");

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

        info!("[TransactionResponderWorker ] Stopped.");
    }
}
