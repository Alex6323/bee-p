use async_std::task::{
    block_on,
    spawn
};

use bee_bundle::{
    Address,
    Hash,
    Transaction,
    TransactionField
};

use bee_crypto::{
    Kerl,
    Sponge
};

use bee_ternary::{
    Trits,
    TritBuf,
    T1B1,
    T5B1,
    T5B1Buf,
    T1B1Buf
};

use crate::protocol::{
    Protocol
};

use crate::message::TransactionBroadcast;
use crate::milestone::MilestoneValidatorWorkerEvent;

use futures::{
    channel::{
        mpsc,
        oneshot,
    },
    future::FutureExt,
    select,
    stream::StreamExt,
    prelude::*,
};

use log::info;

use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::vec_deque::VecDeque;
use crate::ProtocolConfBuilder;

pub(crate) type TransactionWorkerEvent = TransactionBroadcast;

pub(crate) struct TransactionWorker {
    receiver: mpsc::Receiver<TransactionWorkerEvent>,
    shutdown: oneshot::Receiver<()>,
}

impl TransactionWorker {

    pub(crate) fn new(receiver: mpsc::Receiver<TransactionWorkerEvent>, shutdown: oneshot::Receiver<()>) -> Self {

        Self {
            receiver,
            shutdown,
        }

    }

    pub(crate) async fn run(mut self) {

        info!("[TransactionWorker ] Running.");

        let mut receiver_fused = self.receiver.fuse();
        let mut shutdown_fused = self.shutdown.fuse();

        let mut kerl = Kerl::new();
        let mut tangle = TemporaryTangle::new();

        loop {

            let transaction_broadcast: TransactionBroadcast = select! {

                transaction_broadcast = receiver_fused.next() => match transaction_broadcast {

                    Some(transaction_broadcast) => transaction_broadcast,
                    None => break,

                },

                _ = shutdown_fused => break

            };

            info!("[TransactionWorker ] Processing received data...");

            // convert received transaction bytes into T1B1 buffer
            let transaction_buf: TritBuf<T1B1Buf> = {

                // transform &[u8] to &[i8]
                let t5b1_bytes: &[i8] = unsafe { &*(&transaction_broadcast.transaction[..] as *const [u8] as *const [i8]) };

                // get T5B1 trits
                let t5b1_trits: &Trits<T5B1> = Trits::<T5B1>::try_from_raw(t5b1_bytes, 8019).unwrap();

                // get T5B1 trit_buf
                let t5b1_trit_buf: TritBuf<T5B1Buf> = t5b1_trits.to_buf::<T5B1Buf>();

                // get T1B1 trit_buf from TB51 trit_buf
                t5b1_trit_buf.encode::<T1B1Buf>()

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
            let tx_hash: Hash = Hash::from_inner_unchecked(kerl.digest(&transaction_buf).unwrap());

            info!("[TransactionWorker ] Received transaction {}.", tx_hash);

            // check if transactions is already present in the tangle before doing any further work
            if tangle.contains(&tx_hash) {
                info!("[TransactionWorker ] Transaction {} already present in the tangle.", &tx_hash);
                continue;
            }

            // store transaction
            tangle.insert(tx_hash.clone(), built_transaction);
        }

        info!("[TransactionWorker ] Stopped.");

    }
}

struct TemporaryTangle {
    tx_counter: usize,
    capacity: usize,
    tangle: HashMap<Hash, Transaction>,
}

impl TemporaryTangle {
    pub fn new() -> Self {
        Self {
            tx_counter: 0,
            capacity: 10000,
            tangle: HashMap::new()
        }
    }
    pub fn insert(&mut self, hash: Hash, transaction: Transaction) -> bool {
        if self.tx_counter < self.capacity {
            self.tangle.insert(hash.clone(), transaction);
            info!("[Tangle ] Stored transaction {}", &hash);
            self.tx_counter += 1;
            true
        } else {
            info!("[Tangle ] Maximum capacity of the tangle reached, transaction {} can not be stored.", &hash);
            false
        }
    }
    pub fn contains(&self, key: &Hash) -> bool {
        self.tangle.contains_key(key)
    }
    pub fn size(&self) -> usize {
        self.tangle.len()
    }
}

#[test]
fn test_tangle_insert() {

    use bee_bundle::*;

    // create tangle
    let mut tangle = TemporaryTangle::new();

    // build transaction
    let transaction = Transaction::builder()
        .with_payload(Payload::zeros())
        .with_address(Address::zeros())
        .with_value(Value::from_inner_unchecked (0))
        .with_obsolete_tag(Tag::zeros())
        .with_timestamp(Timestamp::from_inner_unchecked(0))
        .with_index(Index::from_inner_unchecked(0))
        .with_last_index(Index::from_inner_unchecked(0))
        .with_tag(Tag::zeros())
        .with_attachment_ts(Timestamp::from_inner_unchecked(0))
        .with_bundle(Hash::zeros())
        .with_trunk(Hash::zeros())
        .with_branch(Hash::zeros())
        .with_attachment_lbts(Timestamp::from_inner_unchecked(0))
        .with_attachment_ubts(Timestamp::from_inner_unchecked(0))
        .with_nonce(Nonce::zeros())
        .build()
        .unwrap();

    // get trits of transaction (using transaction.address())
    let trit_buf: &TritBuf<T1B1Buf> = transaction.address().to_inner();

    // calculate hash of transaction
    let mut kerl = Kerl::new();
    let tx_hash: Hash = Hash::from_inner_unchecked(kerl.digest(&trit_buf).unwrap());

    //store transaction in the tangle
    tangle.insert(tx_hash.clone(), transaction);

    assert_eq!(true, tangle.contains(&tx_hash));

}

#[test]
fn test_tx_worker() {

    let (mut transaction_worker_sender, transaction_worker_receiver) = mpsc::channel(1000);
    let (mut shutdown_sender, shutdown_receiver) = oneshot::channel();

    spawn(async move {
        let tx: [u8; 1604] = [0; 1604];
        let message = TransactionBroadcast::new(&tx);
        transaction_worker_sender.send(message).await.unwrap();
    });

    spawn(async move {
        use std::time::Duration;
        async_std::task::sleep(Duration::from_secs(1)).await;
        shutdown_sender.send(()).unwrap();
    });

    block_on(TransactionWorker::new(transaction_worker_receiver, shutdown_receiver).run());

}

