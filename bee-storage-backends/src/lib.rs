#[macro_use]

extern crate rand;
pub mod sqlx_backend;

#[cfg(test)]




mod tests {
    use crate::sqlx_backend::SqlxBackendStorage;
    use rand::Rng;
    use storage::StorageBackend;
    use futures::executor::block_on;
    use iota_lib_rs::iota_model::Transaction;
    use std::process::Command;
    use std::io::{self, Write};

    fn setup_db() ->() {

        let output = Command::new("schemes/postgress/setup.sh")
            .arg("schemes/postgress/schema.sql")
            .arg("bee_test")
            .arg("dummy_password")
            .output()
            .expect("failed to execute process");

        println!("status: {}", output.status);

        io::stdout().write_all(&output.stdout).unwrap();
        io::stderr().write_all(&output.stderr).unwrap();

        assert!(output.status.success());

    }


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

        setup_db();

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
