use bee_tangle::{tangle, TransactionRef};
use bee_transaction::Hash;

pub trait Api {

    fn transaction_by_hash(hashes: &Hash) -> Option<TransactionRef>;
    fn transactions_by_hash(hashes: &[Hash]) -> Vec<Option<TransactionRef>>;

}

pub struct ApiImpl;

impl Api for ApiImpl {

    fn transaction_by_hash(hash: &Hash) -> Option<TransactionRef> {
        tangle().get_transaction(&hash)
    }

    fn transactions_by_hash(hashes: &[Hash]) -> Vec<Option<TransactionRef>> {
        let mut ret = Vec::new();
        for hash in hashes {
            ret.push(Self::transaction_by_hash(hash));
        }
        ret
    }

}