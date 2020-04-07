use crate::message::TransactionBroadcast;

use bee_bundle::{
    Hash,
    Transaction,
    TransactionField,
};
use bee_crypto::{
    CurlP81,
    Sponge,
};
use bee_tangle::tangle;
use bee_ternary::{
    Error,
    T1B1Buf,
    T5B1Buf,
    TritBuf,
    Trits,
    T5B1,
};

use std::{
    collections::{
        HashSet,
        VecDeque,
    },
    hash::{
        BuildHasherDefault,
        Hasher,
    },
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

use log::info;

use twox_hash::XxHash64;

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
        let mut cache = TinyTransactionCache::new(10000);

        loop {
            let transaction_broadcast: TransactionBroadcast = select! {

                transaction_broadcast = receiver_fused.next() => match transaction_broadcast {

                    Some(transaction_broadcast) => transaction_broadcast,
                    None => {
                        info!("[TransactionWorker ] Unable to receive transactions from channel.");
                        break;
                    },

                },

                _ = shutdown_fused => break

            };

            info!("[TransactionWorker ] Processing received data...");

            if !cache.insert(xx_hash(transaction_broadcast.transaction.as_slice())) {
                info!("[TransactionWorker ] Data already received.");
                continue;
            }

            // convert received transaction bytes into T1B1 buffer
            let transaction_buf: TritBuf<T1B1Buf> = {
                // transform &[u8] to &[i8]
                let t5b1_bytes: &[i8] =
                    unsafe { &*(transaction_broadcast.transaction.as_slice() as *const [u8] as *const [i8]) };

                // get T5B1 trits
                let t5b1_trits_result: Result<&Trits<T5B1>, Error> =
                    Trits::<T5B1>::try_from_raw(t5b1_bytes, t5b1_bytes.len() * 5 - 1);

                match t5b1_trits_result {
                    Ok(t5b1_trits) => {
                        // get T5B1 trit_buf
                        let t5b1_trit_buf: TritBuf<T5B1Buf> = t5b1_trits.to_buf::<T5B1Buf>();

                        // get T1B1 trit_buf from TB51 trit_buf
                        t5b1_trit_buf.encode::<T1B1Buf>()
                    }
                    Err(_) => {
                        info!("[TransactionWorker ] Can not decode T5B1 from received data.");
                        continue;
                    }
                }
            };

            // build transaction
            let transaction_result = Transaction::from_trits(&transaction_buf);

            // validate transaction result
            let built_transaction = match transaction_result {
                Ok(tx) => tx,
                Err(_) => {
                    info!("[TransactionWorker ] Can not build transaction from received data.");
                    continue;
                }
            };

            // calculate transaction hash
            let tx_hash: Hash = Hash::from_inner_unchecked(curl.digest(&transaction_buf).unwrap());

            info!("[TransactionWorker ] Received transaction {}.", &tx_hash);

            // check if transactions is already present in the tangle before doing any further work
            if tangle().contains_transaction(&tx_hash) {
                info!(
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

fn xx_hash(buf: &[u8]) -> u64 {
    let mut hasher = XxHash64::default();

    hasher.write(buf);
    hasher.finish()
}

struct CustomHasher {
    result: Option<u64>,
}

impl CustomHasher {
    fn finish(&self) -> u64 {
        self.result.unwrap()
    }
    fn write(&mut self, i: u64) {
        self.result = Some(i);
    }
}

impl Default for CustomHasher {
    fn default() -> Self {
        Self { result: None }
    }
}

impl Hasher for CustomHasher {
    fn finish(&self) -> u64 {
        CustomHasher::finish(self)
    }

    fn write(&mut self, _bytes: &[u8]) {
        unreachable!();
    }

    fn write_u64(&mut self, i: u64) {
        CustomHasher::write(self, i);
    }
}

struct TinyTransactionCache {
    max_capacity: usize,
    cache: HashSet<u64, BuildHasherDefault<CustomHasher>>,
    order: VecDeque<u64>,
}

impl TinyTransactionCache {
    pub fn new(max_capacity: usize) -> Self {
        Self {
            max_capacity,
            cache: HashSet::default(),
            order: VecDeque::new(),
        }
    }

    pub fn insert(&mut self, hash: u64) -> bool {
        if self.contains(&hash) {
            return false;
        }

        if self.cache.len() >= self.max_capacity {
            let first = self.order.pop_front().unwrap();
            self.cache.remove(&first);
        }

        self.cache.insert(hash.clone());
        self.order.push_back(hash);

        true
    }

    fn contains(&self, hash: &u64) -> bool {
        self.cache.contains(hash)
    }

    pub fn len(&self) -> usize {
        self.cache.len()
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
    fn test_cache_insert() {
        let mut cache = TinyTransactionCache::new(10);

        let first_buf = &[1, 2, 3];
        let second_buf = &[1, 2, 3];

        assert_eq!(cache.insert(xx_hash(first_buf)), true);
        assert_eq!(cache.insert(xx_hash(second_buf)), false);
    }

    #[test]
    fn test_cache_max_capacity() {
        let mut cache = TinyTransactionCache::new(1);

        let first_buf = &[1, 2, 3];
        let second_buf = &[3, 4, 5];
        let second_buf_clone = second_buf.clone();

        assert_eq!(cache.insert(xx_hash(first_buf)), true);
        assert_eq!(cache.insert(xx_hash(second_buf)), true);
        assert_eq!(cache.len(), 1);
        assert_eq!(cache.insert(xx_hash(&second_buf_clone)), false);
    }

    #[test]
    fn test_tx_worker() {
        bee_tangle::init();

        let (mut transaction_worker_sender, transaction_worker_receiver) = mpsc::channel(1000);
        let (mut shutdown_sender, shutdown_receiver) = oneshot::channel();

        spawn(async move {
            let tx: [u8; 1604] = [0; 1604];
            let message = TransactionBroadcast::new(&tx);
            transaction_worker_sender.send(message).await.unwrap();
        });

        spawn(async move {
            use async_std::task;
            use std::time::Duration;
            task::sleep(Duration::from_secs(2)).await;
            shutdown_sender.send(()).unwrap();
        });

        block_on(TransactionWorker::new().run(transaction_worker_receiver, shutdown_receiver));

        assert_eq!(tangle().contains_transaction(&Hash::zeros()), true);
    }
}
