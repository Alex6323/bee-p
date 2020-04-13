use crate::{
    message::TransactionBroadcast,
    worker::transaction::TinyHashCache,
};

use bee_bundle::{
    Hash,
    Transaction,
    TransactionField,
};
use bee_common::constants::TRANSACTION_BYTE_LEN;
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

pub(crate) struct TransactionWorker {
    cache_size: usize,
}

impl TransactionWorker {
    pub(crate) fn new(cache_size: usize) -> Self {
        Self { cache_size }
    }

    pub(crate) async fn run(self, receiver: mpsc::Receiver<TransactionWorkerEvent>, shutdown: oneshot::Receiver<()>) {
        info!("[TransactionWorker ] Running.");

        let mut receiver_fused = receiver.fuse();
        let mut shutdown_fused = shutdown.fuse();

        let mut curl = CurlP81::new();
        let mut cache = TinyHashCache::new(self.cache_size);

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
                // define max buffer and copy received transaction bytes into it
                let mut u8_t5b1_buf = [0u8; TRANSACTION_BYTE_LEN];
                // NOTE: following copying relies on validly sized input data
                u8_t5b1_buf[..transaction_broadcast.transaction.len()]
                    .copy_from_slice(&transaction_broadcast.transaction);

                // transform [u8] to &[i8]
                let i8_t5b1_slice = unsafe { &*(&u8_t5b1_buf as *const [u8] as *const [i8]) };

                // get T5B1 trits
                let t5b1_trits_result = Trits::<T5B1>::try_from_raw(i8_t5b1_slice, i8_t5b1_slice.len() * 5 - 1);

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
            let transaction = match Transaction::from_trits(&transaction_buf) {
                Ok(transaction) => transaction,
                Err(e) => {
                    warn!(
                        "[TransactionWorker ] Can not build transaction from received data: {:?}",
                        e
                    );
                    continue;
                }
            };

            // calculate transaction hash
            let hash = Hash::from_inner_unchecked(curl.digest(&transaction_buf).unwrap());

            // store transaction
            match tangle().insert_transaction(transaction, hash).await {
                Some(_) => {}
                None => {
                    debug!(
                        "[TransactionWorker ] Transaction {} already present in the tangle.",
                        &hash
                    );
                }
            }
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

        block_on(TransactionWorker::new().run(transaction_worker_receiver, shutdown_receiver, 10000));

        assert_eq!(tangle().size(), 1);
        assert_eq!(tangle().contains_transaction(&Hash::zeros()), true);
    }
}
