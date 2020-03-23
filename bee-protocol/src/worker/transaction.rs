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

use futures::channel::mpsc;
use futures::channel::mpsc::{channel, SendError, Sender, Receiver};
use futures::channel::oneshot;
use futures::stream::StreamExt;
use futures::prelude::*;
use futures::select;

use log::info;

use std::collections::HashMap;
use crate::worker::milestone_validator::MilestoneValidatorWorkerEvent;

pub enum TransactionWorkerEvent {
    Transaction(TransactionBroadcast),
}

pub struct TransactionWorker {
    receiver: Receiver<TransactionWorkerEvent>,
    milestone_validator_worker_sender: Sender<MilestoneValidatorWorkerEvent>
}

impl TransactionWorker {

    pub fn new(receiver: Receiver<TransactionWorkerEvent>, milestone_validator_worker_sender: Sender<MilestoneValidatorWorkerEvent>) -> Self {
        Self {
            receiver,
            milestone_validator_worker_sender
        }
    }

    pub async fn run(mut self) {

        info!("[TransactionWorker ] Running.");

        let mut tangle = TemporaryTangle::new();

        let receiver = &mut self.receiver;
        let milestone_validator_worker_sender = &mut self.milestone_validator_worker_sender;
        //let shutdown_receiver = &mut self.shutdown_receiver;

        loop {

            select! {

                transaction_broadcast = receiver.next().fuse() => match transaction_broadcast {

                    Some(TransactionWorkerEvent::Transaction(TransactionBroadcast { transaction } )) => {

                        info!("[TransactionWorker ] Processing received data...");

                        // transform &[u8] to &[i8]
                        let i8_transaction_slice = unsafe { &*(&transaction[..] as *const [u8] as *const [i8]) };

                        // get T5B1 trits
                        let t5b1_trits: &Trits<T5B1> = Trits::<T5B1>::try_from_raw(i8_transaction_slice).unwrap();

                        // get T5B1 trit_buf
                        let t5b1_trit_buf = t5b1_trits.to_buf::<T5B1Buf>();

                        // get T1B1 trit_buf from TB51 trit_buf
                        let t1b1_trit_buf = t5b1_trit_buf.encode::<T1B1Buf>();

                        // build transaction
                        let transaction_result = Transaction::from_trits(&t5b1_trit_buf);

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

                        // store transaction
                        tangle.insert(tx_hash.clone(), built_transaction);

                       //milestone_validator_worker_sender.send(MilestoneValidatorWorkerEvent::Candidate{0: t1b1_trit_buf}).await.unwrap();

                    },

                    None => {
                        info!("[TransactionWorker ] Unable to read transactions from receiver.");
                        break;
                    }

                },

                //shutdown = shutdown_receiver.fuse() => {
                //    info!("[TransactionWorker ] Terminating TransactionWorker...");
                //    break;
                //}

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

    // TODO: something like Transaction::to_trits() would be helpful to convert Transaction with its fields to a T1B1-trit-buf
    // calculate hash of transaction
    //let mut curlp27 = CurlP27::new();
    //let tx_hash: Hash = Hash { 0: curlp27.digest(&t1b1_trit_buf).unwrap() };

    // store transaction in the tangle
    //tangle.insert(tx_hash.clone(), transaction);

    //assert_eq!(true, tangle.contains(&tx_hash));

}

