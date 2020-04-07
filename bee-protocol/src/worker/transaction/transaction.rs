use crate::{
    message::TransactionBroadcast,
    worker::transaction::TinyHashCache,
};

use bee_bundle::{
    Hash,
    Transaction,
    TransactionField,
};
use bee_common::constants;
use bee_crypto::{
    CurlP81,
    Sponge,
};
use bee_tangle::tangle;
use bee_ternary::{
    T1B1Buf,
    T5B1Buf,
    Trits,
    T5B1,
};

use futures::{
    channel::{
        mpsc,
        oneshot,
    },
    future::FutureExt,
    select,
    stream::StreamExt,
};
use log::{
    debug,
    info,
    warn,
};

pub(crate) type TransactionWorkerEvent = TransactionBroadcast;

pub(crate) struct TransactionWorker;

impl TransactionWorker {
    pub(crate) fn new() -> Self {
        Self {}
    }

    pub(crate) async fn run(self, receiver: mpsc::Receiver<TransactionWorkerEvent>, shutdown: oneshot::Receiver<()>) {
        info!("[TransactionWorker ] Running.");

        let mut receiver_fused = receiver.fuse();
        let mut shutdown_fused = shutdown.fuse();

        let mut curl = CurlP81::new();
        // TODO conf
        let mut cache = TinyHashCache::new(10000);

        loop {
            let transaction_broadcast = select! {

                transaction_broadcast = receiver_fused.next() => match transaction_broadcast {

                    Some(transaction_broadcast) => transaction_broadcast,
                    None => {
                        debug!("[TransactionWorker ] Unable to receive transactions from channel.");
                        break;
                    },

                },

                _ = shutdown_fused => break

            };

            debug!("[TransactionWorker ] Processing received data...");

            if !cache.insert(transaction_broadcast.transaction.as_slice()) {
                debug!("[TransactionWorker ] Data already received.");
                continue;
            }

            // convert received transaction bytes into T1B1 buffer
            let transaction_buf = {
                let mut raw_bytes = transaction_broadcast.transaction;
                while raw_bytes.len() < constants::TRANSACTION_BYTE_LEN {
                    raw_bytes.push(0);
                }

                // transform &[u8] to &[i8]
                let t5b1_bytes: &[i8] = unsafe { &*(raw_bytes.as_slice() as *const [u8] as *const [i8]) };

                // get T5B1 trits
                let t5b1_trits_result = Trits::<T5B1>::try_from_raw(t5b1_bytes, t5b1_bytes.len() * 5 - 1);

                match t5b1_trits_result {
                    Ok(t5b1_trits) => {
                        // get T5B1 trit_buf
                        let t5b1_trit_buf = t5b1_trits.to_buf::<T5B1Buf>();

                        // get T1B1 trit_buf from TB51 trit_buf
                        t5b1_trit_buf.encode::<T1B1Buf>()
                    }
                    Err(_) => {
                        warn!("[TransactionWorker ] Can not decode T5B1 from received data.");
                        continue;
                    }
                }
            };

            // build transaction
            let built_transaction = match Transaction::from_trits(&transaction_buf) {
                Ok(tx) => tx,
                Err(_) => {
                    warn!("[TransactionWorker ] Can not build transaction from received data.");
                    continue;
                }
            };

            // calculate transaction hash
            let tx_hash = Hash::from_inner_unchecked(curl.digest(&transaction_buf).unwrap());

            debug!("[TransactionWorker ] Received transaction {}.", &tx_hash);

            // check if transactions is already present in the tangle before doing any further work
            if tangle().contains_transaction(&tx_hash) {
                debug!(
                    "[TransactionWorker ] Transaction {} already present in the tangle.",
                    &tx_hash
                );
                continue;
            }

            // store transaction
            tangle().insert_transaction(built_transaction, tx_hash).await;
        }

        info!("[TransactionWorker ] Stopped.");
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    use async_std::task::{
        block_on,
        spawn,
    };
    use futures::sink::SinkExt;

    #[test]
    fn test_tx_worker_with_compressed_buffer() {
        bee_tangle::init();

        assert_eq!(tangle().size(), 0);

        let (transaction_worker_sender, transaction_worker_receiver) = mpsc::channel(1000);
        let (mut shutdown_sender, shutdown_receiver) = oneshot::channel();

        let mut transaction_worker_sender_clone = transaction_worker_sender.clone();
        spawn(async move {
            let tx: [u8; 1024] = [0; 1024];
            let message = TransactionBroadcast::new(&tx);
            transaction_worker_sender_clone.send(message).await.unwrap();
        });

        spawn(async move {
            use async_std::task;
            use std::time::Duration;
            task::sleep(Duration::from_secs(1)).await;
            shutdown_sender.send(()).unwrap();
        });

        block_on(TransactionWorker::new().run(transaction_worker_receiver, shutdown_receiver));

        assert_eq!(tangle().size(), 1);
        assert_eq!(tangle().contains_transaction(&Hash::zeros()), true);
    }
}
