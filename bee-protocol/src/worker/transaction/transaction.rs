use crate::{
    message::TransactionBroadcast,
    protocol::Protocol,
    util::uncompress_transaction_bytes,
    worker::transaction::TinyHashCache,
};

use bee_bundle::{
    Hash,
    Transaction,
    TransactionField,
};
use bee_crypto::{
    CurlP81,
    Sponge,
};
use bee_network::EndpointId;
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

pub(crate) struct TransactionWorkerEvent {
    pub(crate) from: EndpointId,
    pub(crate) transaction_broadcast: TransactionBroadcast,
}

pub(crate) struct TransactionWorker {
    cache: TinyHashCache,
    curl: CurlP81,
}

impl TransactionWorker {
    pub(crate) fn new(cache_size: usize) -> Self {
        Self {
            cache: TinyHashCache::new(cache_size),
            curl: CurlP81::new(),
        }
    }

    pub(crate) async fn run(
        mut self,
        receiver: mpsc::Receiver<TransactionWorkerEvent>,
        shutdown: oneshot::Receiver<()>,
    ) {
        info!("[TransactionWorker ] Running.");

        let mut receiver_fused = receiver.fuse();
        let mut shutdown_fused = shutdown.fuse();

        loop {
            select! {
                event = receiver_fused.next() => {
                    if let Some(TransactionWorkerEvent{from, transaction_broadcast}) = event {
                        self.process_transaction_brodcast(from, transaction_broadcast).await;
                    }
                },
                _ = shutdown_fused => break

            }
        }

        info!("[TransactionWorker ] Stopped.");
    }

    async fn process_transaction_brodcast(&mut self, from: EndpointId, transaction_broadcast: TransactionBroadcast) {
        debug!("[TransactionWorker ] Processing received data...");

        if !self.cache.insert(&transaction_broadcast.transaction) {
            debug!("[TransactionWorker ] Data already received.");
            return;
        }

        // convert received transaction bytes into T1B1 buffer
        let transaction_buf = {
            let u8_t5b1_buf = uncompress_transaction_bytes(&transaction_broadcast.transaction);

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
                    return;
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
                return;
            }
        };

        // calculate transaction hash
        let hash = Hash::from_inner_unchecked(self.curl.digest(&transaction_buf).unwrap());

        if hash.weight() < Protocol::get().conf.mwm {
            debug!("[TransactionWorker ] Insufficient weight magnitude: {}.", hash.weight());
            return;
        }

        // store transaction
        match tangle().insert_transaction(transaction, hash).await {
            Some(_) => Protocol::broadcast_transaction_message(Some(from), transaction_broadcast).await,
            None => {
                debug!(
                    "[TransactionWorker ] Transaction {} already present in the tangle.",
                    &hash
                );
            }
<<<<<<< bfd61b982745cdab1f743e114311e71aa5f54c4d
=======

            // store transaction
            tangle().insert_transaction(built_transaction, tx_hash);
>>>>>>> Implement 'get_transaction' API
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    use bee_network::Address;

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
            transaction_worker_sender_clone
                .send(TransactionWorkerEvent {
                    from: EndpointId::from(block_on(Address::from_addr_str("127.0.0.1:1337")).unwrap()),
                    transaction_broadcast: message,
                })
                .await
                .unwrap();
        });

        spawn(async move {
            use async_std::task;
            use std::time::Duration;
            task::sleep(Duration::from_secs(1)).await;
            shutdown_sender.send(()).unwrap();
        });

        block_on(TransactionWorker::new(10000).run(transaction_worker_receiver, shutdown_receiver));

        assert_eq!(tangle().size(), 1);
        assert_eq!(tangle().contains_transaction(&Hash::zeros()), true);
    }
}
