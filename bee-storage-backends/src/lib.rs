#[macro_use]

extern crate rand;
pub mod sqlx_backend;
pub use bundle::*;

#[cfg(test)]




mod tests {


    //FIXME - figure out (even when only one test with one thread):
    //psql:cleanup.sql:1: ERROR:  database "test_db" is being accessed by other users
    //DETAIL:  There is 1 other session using the database.


    const BEE_TEST_DB_USER: &str = "test_db_user";
    const BEE_TEST_DB_NAME: &str = "test_db";

    use crate::sqlx_backend::SqlxBackendStorage;
    use rand::Rng;
    use storage::{Milestone, StorageBackend};
    use futures::executor::block_on;
    use iota_lib_rs::iota_model::{Transaction};
    use std::process::Command;
    use std::io::{self, Write};
    use std::panic;

    fn rand_hash_string() -> bundle::Hash{
        use rand::Rng;
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ9";
        const HASH_LEN: usize = 81;
        let mut rng = rand::thread_rng();

        let hash_str : String = (0..HASH_LEN)
            .map(|_| {
                let idx = rng.gen_range(0, CHARSET.len());
                CHARSET[idx] as char
            })
            .collect();

        let hash: bundle::Hash = hash_str.into();
        hash
    }

    fn create_random_tx() -> (bundle::Hash, bundle::Transaction) {
        let mut builder = bundle::TransactionBuilder::default();
        builder
            .value(bundle::Value(10))
            .address(bundle::Address::from_str("ME"))
            .tag(bundle::Tag::from_str("HELLO"))
            .nonce(bundle::Nonce::from_str("ABCDEF"));

        (rand_hash_string() , builder.build())
    }

    fn create_random_milestone() -> Milestone {
        Milestone {
            hash: rand_hash_string().to_string(),
            index: 0,
        }
    }

    fn run_test<T>(test: T) -> ()
        where T: FnOnce() -> () + panic::UnwindSafe
    {
        setup_db();

        let result = panic::catch_unwind(|| {
            test()
        });

        teardown_db();

        assert!(result.is_ok())

    }

    fn setup_db() ->() {

        let output = Command::new("schemes/postgress/setup.sh")
            .arg("schemes/postgress/schema.sql")
            .arg(BEE_TEST_DB_USER)
            .arg("dummy_password")
            .arg(BEE_TEST_DB_NAME)
            .output()
            .expect("failed to execute setup process");

        println!("status: {}", output.status);

        io::stdout().write_all(&output.stdout).unwrap();
        io::stderr().write_all(&output.stderr).unwrap();

        assert!(output.status.success());

    }

    fn teardown_db() ->() {

        let output = Command::new("schemes/postgress/cleanup.sh")
            .arg(BEE_TEST_DB_USER)
            .arg(BEE_TEST_DB_NAME)
            .output()
            .expect("failed to execute cleanup process");

        io::stdout().write_all(b"TEARING DOWN").unwrap();

        println!("status: {}", output.status);

        io::stdout().write_all(&output.stdout).unwrap();
        io::stderr().write_all(&output.stderr).unwrap();

        assert!(output.status.success());

    }

    #[test]
    fn test_insert_one_transaction() {
        run_test(|| {
            let mut storage = SqlxBackendStorage::new();

            block_on(storage.establish_connection());
            let (tx_hash, tx) = create_random_tx();
            block_on(storage.insert_transaction(&tx_hash, &tx));
            let res = block_on(storage.find_transaction(&tx_hash));
            let found_tx = res.unwrap();
            block_on(storage.destroy_connection());
            assert_eq!(tx.nonce.0, found_tx.nonce.0);
        })
    }

    #[test]
    fn test_insert_one_milestone() {
        run_test(|| {
            let mut storage = SqlxBackendStorage::new();

            block_on(storage.establish_connection());
            let mut milestone = create_random_milestone();
            milestone.index = 1;
            block_on(storage.insert_milestone(&milestone));
            let res = block_on(storage.find_milestone(milestone.hash.as_str()));
            let found_milestone = res.unwrap();
            block_on(storage.destroy_connection());
            assert_eq!(milestone, found_milestone);
        })
    }
}
