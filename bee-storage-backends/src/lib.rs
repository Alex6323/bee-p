#[macro_use]

extern crate rand;

pub mod sqlx_backend;

use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;



#[cfg(test)]




mod tests {
    use crate::sqlx_backend::SqlxBackendStorage;
    use rand::Rng;
    use storage::StorageBackend;
    use futures::executor::block_on;
    use iota_lib_rs::iota_model::Transaction;


    fn rand_hash_string() -> String{
        use rand::Rng;
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ9";
        const HASH_LEN: usize = 81;
        let mut rng = rand::thread_rng();

        (0..HASH_LEN)
            .map(|_| {
                let idx = rng.gen_range(0, CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn insert_one_transaction(){

        let mut storage = SqlxBackendStorage::new();

        block_on(storage.establish_connection());

        let mut tx = Transaction {
            hash: rand_hash_string(),
            tag: String::from("tag"),
            bundle: String::from("bundle"),
            address: String::from("address"),
            trunk_transaction: rand_hash_string(),
            branch_transaction: rand_hash_string(),
            nonce: String::from("nonce"),
            attachment_timestamp_lower_bound: 1,
            attachment_timestamp_upper_bound: 10,
            attachment_timestamp: 6,
            signature_fragments: String::from("signature_fragment"),
            current_index: 0,
            last_index: 1,
            persistence: true,
            timestamp: 100,
            value: -100,
            obsolete_tag: String::from("obsolete_tag"),
        };

        block_on(storage.insert_transaction(&tx));
        let res = block_on(storage.find_transaction(tx.hash.as_str()));
        let found_tx = res.unwrap();
        assert_eq!(tx, found_tx);
    }
}
