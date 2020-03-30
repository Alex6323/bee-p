use async_std::task::{block_on, spawn};

use bee_bundle::Hash;
use bee_bundle::Transaction;

use bee_crypto::CurlP27;
use bee_crypto::Sponge;

use bee_ternary::Trits;
use bee_ternary::TritBuf;
use bee_ternary::T1B1;
use bee_ternary::T5B1;
use bee_ternary::T5B1Buf;
use bee_ternary::T1B1Buf;

use crate::message::TransactionBroadcast;
use crate::protocol::{
    Protocol,
    COORDINATOR_BYTES
};
use crate::milestone::MilestoneValidatorWorkerEvent;

use futures::{
    channel::mpsc::{channel, SendError, Sender, Receiver},
    stream::StreamExt,
    prelude::*,
    select,
};
use log::info;

use std::collections::HashMap;

pub(crate) type TransactionWorkerEvent = TransactionBroadcast;

pub(crate) struct TransactionWorker {
    receiver: Receiver<TransactionWorkerEvent>,
    milestone_validator_worker_sender: Sender<MilestoneValidatorWorkerEvent>
}

impl TransactionWorker {
    pub(crate) fn new(receiver: Receiver<TransactionWorkerEvent>) -> Self {
        Self {
            receiver: receiver,
            milestone_validator_worker_sender: Protocol::get().milestone_validator_worker.clone(),
        }
    }

    pub(crate) async fn run(mut self) {
        info!("[TransactionWorker ] Running.");

        let mut tangle = TemporaryTangle::new();

        let receiver = &mut self.receiver;
        //let milestone_validator_worker_sender = &mut self.milestone_validator_worker_sender;
        //let shutdown_receiver = &mut self.shutdown_receiver;

        while let Some(transaction_broadcast) = self.receiver.next().await {

            // convert bytes of coordinator address to i8 slice
            let t5b1_coo_i8 = unsafe { &*(&COORDINATOR_BYTES[..] as *const [u8] as *const [i8]) };
            let t1b1_coo_buf = Trits::<T5B1>::try_from_raw(t5b1_coo_i8,243).unwrap().to_buf::<T1B1Buf>();

            info!("[TransactionWorker ] Processing received data...");

            let transaction = transaction_broadcast.transaction;

            // transform &[u8] to &[i8]
            let t5b1_transaction: &[i8] = unsafe { &*(&transaction[..] as *const [u8] as *const [i8]) };

            // get T5B1 trits
            let t5b1_trits: &Trits<T5B1> = Trits::<T5B1>::try_from_raw(t5b1_transaction, 8019).unwrap();

            // get T5B1 trit_buf
            let t5b1_trit_buf: TritBuf<T5B1Buf> = t5b1_trits.to_buf::<T5B1Buf>();

            // get T1B1 trit_buf from TB51 trit_buf
            let t1b1_trit_buf: TritBuf<T1B1Buf> = t5b1_trit_buf.encode::<T1B1Buf>();

            // build transaction
            let transaction_result = Transaction::from_trits(&t1b1_trit_buf);

            // validate transaction result
            let built_transaction = match transaction_result {
                Ok(tx) => tx,
                Err(_) => {
                    info!("[TransactionWorker ] Can not build transaction from received data.");
                    continue;
                }
            };

            // calculate transaction hash
            let mut curlp27 = CurlP27::new();
            let tx_hash: Hash = Hash { 0: curlp27.digest(&t1b1_trit_buf).unwrap() };

            info!("[TransactionWorker ] Received transaction {}.", tx_hash);

            // check if transactions is already present in the tangle before doing any further work
            if tangle.contains(&tx_hash) {
                info!("[TransactionWorker ] Transaction {} already present in the tangle.", &tx_hash);
                continue;
            }

            // get address of transaction
            let address = built_transaction.address().clone();

            // store transaction
            tangle.insert(tx_hash.clone(), built_transaction);

            // check if transaction is a potential milestone candidate
            if address.0.eq(&t1b1_coo_buf) {
                //milestone_validator_worker_sender.send(MilestoneValidatorWorkerEvent::Candidate(tx_hash.0)).await.unwrap();
            }

        }
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
        .with_value(Value(0))
        .with_obsolete_tag(Tag::zeros())
        .with_timestamp(Timestamp(0))
        .with_index(Index(0))
        .with_last_index(Index(0))
        .with_tag(Tag::zeros())
        .with_attachment_ts(Timestamp(0))
        .with_bundle(Hash::zeros())
        .with_trunk(Hash::zeros())
        .with_branch(Hash::zeros())
        .with_attachment_lbts(Timestamp(0))
        .with_attachment_ubts(Timestamp(0))
        .with_nonce(Nonce::zeros())
        .build()
        .unwrap();

    // get trits of transaction (using transaction.address())
    let trit_buf: TritBuf<T1B1Buf> = transaction.address().into_inner();

    // calculate hash of transaction
    let mut curlp27 = CurlP27::new();
    let tx_hash: Hash = Hash { 0: curlp27.digest(&trit_buf).unwrap() };

    //store transaction in the tangle
    tangle.insert(tx_hash.clone(), transaction);

    assert_eq!(true, tangle.contains(&tx_hash));

}

#[test]
fn test_identify_coo_address() {

    use bee_bundle::*;

    // convert bytes of coordinator address to i8 slice
    let t5b1_coo: &[i8] = unsafe { &*(&COORDINATOR_BYTES[..] as *const [u8] as *const [i8]) };
    //
    let t5b1_coo_buf: TritBuf<T5B1Buf> = Trits::<T5B1>::try_from_raw(t5b1_coo, 243).unwrap().to_buf();

    let t1b1_coo_buf: TritBuf<T1B1Buf> = t5b1_coo_buf.encode::<T1B1Buf>();

    // build address
    let address: Address = Address::try_from_tritbuf(t1b1_coo_buf).unwrap();

    assert_eq!(243, address.as_bytes().len());

}

#[test]
fn test_tx_worker() {

    Protocol::init();

    //let (milestone_validator_worker_sender, milestone_validator_worker_receiver) = channel(1000);
    let (transaction_worker_sender, transaction_worker_receiver) = channel(1000);

    // send tx to the channel
    block_on(tx_sender(transaction_worker_sender));

    block_on(TransactionWorker::new(transaction_worker_receiver).run());
}

async fn tx_sender(mut sender: Sender<TransactionWorkerEvent>) {

    Protocol::init();
    let tx: [u8; 1604] = [0; 1604];
    let message = TransactionBroadcast::new(&tx);
    sender.send(message).await.unwrap();
}

