use bee_tangle::{tangle, TransactionRef};
use bee_transaction::{Hash, BundledTransaction};
use async_trait::async_trait;
use std::sync::Arc;

#[async_trait]
pub trait Api {

    async fn transactions_by_hash(hashes: &[Hash]) -> Vec<TransactionRef>;

}

pub struct ApiImpl;

#[async_trait]
impl Api for ApiImpl {

    async fn transactions_by_hash(hashes: &[Hash]) -> Vec<TransactionRef> {
        let mut ret = Vec::new();
        for hash in hashes {
            match tangle().get_transaction(hash) {
                Some(tx_ref) => ret.push(tx_ref),
                None => continue
            }
        }
        ret
    }

}