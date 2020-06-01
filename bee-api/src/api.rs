use bee_tangle::{tangle, TransactionRef};
use bee_transaction::Hash;

pub trait Api {

    fn transactions_by_hash(hashes: &[Hash]) -> Vec<Option<TransactionRef>>;

}

pub struct ApiImpl;

impl Api for ApiImpl {

    fn transactions_by_hash(hashes: &[Hash]) -> Vec<Option<TransactionRef>> {
        let mut ret = Vec::new();
        for hash in hashes {
            ret.push(tangle().get_transaction(hash));
        }
        ret
    }

}