use bee_bundle::{Hash, Transaction, TransactionField};

pub trait Api {

    fn find_transaction(hash: Hash) -> Option<Transaction>;
    fn find_transactions_by_hash(hashes: &[Hash]) -> Vec<Transaction>;

}

pub struct ApiImpl;

impl Api for ApiImpl {

    fn find_transaction(hash: Hash) -> Option<Transaction> {
        unimplemented!()
    }

    fn find_transactions_by_hash(hashes: &[Hash]) -> Vec<Transaction> {
        unimplemented!()
    }

}