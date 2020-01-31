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
    use std::process::Command;
    use std::io::{self, Write};
    use std::panic;
    use std::borrow::Borrow;
    use bundle::Hash;
    use std::collections::HashSet;
    use std::rc::Rc;

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

        Hash::from_str(&hash_str)
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

    fn create_random_attached_tx(branch: bundle::Hash, trunk: bundle::Hash) -> (bundle::Hash, bundle::Transaction) {
        let mut builder = bundle::TransactionBuilder::default();
        builder
            .branch_hash(branch)
            .trunk_hash(trunk)
            .value(bundle::Value(10))
            .address(bundle::Address::from_str("ME"))
            .tag(bundle::Tag::from_str("HELLO"))
            .nonce(bundle::Nonce::from_str("ABCDEF"));


        (rand_hash_string() , builder.build())
    }

    fn create_random_milestone() -> Milestone {
        Milestone {
            hash: rand_hash_string(),
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
            assert_eq!(tx.nonce().0, found_tx.nonce().0);
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
            let res = block_on(storage.find_milestone(milestone.hash.borrow()));
            let found_milestone = res.unwrap();
            block_on(storage.destroy_connection());
            assert_eq!(milestone.hash.to_string(), found_milestone.hash.to_string());
        })
    }


        #[test]
        fn test_delete_one_transaction() {
            run_test(|| {
                let mut storage = SqlxBackendStorage::new();

                block_on(storage.establish_connection());
                let (tx_hash, tx) = create_random_tx();
                block_on(storage.insert_transaction(&tx_hash, &tx));
                let res = block_on(storage.find_transaction(&tx_hash));
                let found_tx = res.unwrap();
                assert_eq!(tx.nonce().0, found_tx.nonce().0);
                let mut transactions_to_delete = HashSet::new();
                transactions_to_delete.insert(tx_hash);
                let res = block_on(storage.delete_transactions(&transactions_to_delete));
                assert!(res.is_ok());
                let res = block_on(storage.find_transaction(&transactions_to_delete.iter().last().unwrap()));
                block_on(storage.destroy_connection());
                assert!(res.is_err());
            })
        }

    #[test]
    fn test_delete_one_milestone() {
        run_test(|| {
            let mut storage = SqlxBackendStorage::new();

            block_on(storage.establish_connection());
            let mut milestone = create_random_milestone();
            milestone.index = 2;
            block_on(storage.insert_milestone(&milestone));
            let res = block_on(storage.find_milestone(milestone.hash.borrow()));
            let found_milestone = res.unwrap();
            assert_eq!(milestone.hash.to_string(), found_milestone.hash.to_string());
            let mut milestones_to_delete = HashSet::new();
            milestones_to_delete.insert(milestone.hash);
            let res = block_on(storage.delete_milestones(&milestones_to_delete));
            assert!(res.is_ok());
            let res = block_on(storage.find_milestone(&milestones_to_delete.iter().last().unwrap()));
            block_on(storage.destroy_connection());
            assert!(res.is_err());

        })
    }


    #[test]
    fn test_transaction_multiple_delete() {
        run_test(|| {
            let mut storage = SqlxBackendStorage::new();

            block_on(storage.establish_connection());

            let mut hashes = HashSet::new();

            for i in 0 .. 10 {
                let (tx_hash, tx) = create_random_tx();
                block_on(storage.insert_transaction(&tx_hash, &tx));
                let res = block_on(storage.find_transaction(&tx_hash));
                let found_tx = res.unwrap();
                assert_eq!(tx.nonce().0, found_tx.nonce().0);
                hashes.insert(tx_hash);
            }

            let res = block_on(storage.delete_transactions(&hashes));
            assert!(res.is_ok());

            for hash in hashes.iter() {
                let res = block_on(storage.find_transaction(&hash));
                assert!(res.is_err())
            }

            block_on(storage.destroy_connection());
        })
    }


    #[test]
    fn test_map_hashes_to_approvers() {
        run_test(|| {
            let mut storage = SqlxBackendStorage::new();

            block_on(storage.establish_connection());

            let mut hash_to_approvers_expected = storage::HashesToApprovers::new();
            let (tx_hash, tx) =  create_random_tx();
            block_on(storage.insert_transaction(&tx_hash, &tx));
            let res = block_on(storage.find_transaction(&tx_hash));
            let found_tx = res.unwrap();
            assert_eq!(tx.nonce().0, found_tx.nonce().0);
            let mut last_tx_hash = tx_hash.clone();

            for i in 0 .. 2000 {
                let (tx_hash, mut tx) = create_random_attached_tx(last_tx_hash.clone(), last_tx_hash.clone());
                let mut approvers  = HashSet::new();
                approvers.insert(tx_hash.clone());
                hash_to_approvers_expected.insert(last_tx_hash.clone(), approvers);
                block_on(storage.insert_transaction(&tx_hash, &tx));
                let res = block_on(storage.find_transaction(&tx_hash));
                let found_tx = res.unwrap();
                assert_eq!(tx.nonce().0, found_tx.nonce().0);
                last_tx_hash = tx_hash.clone();
            }

            let hash_to_approvers_observed = storage.map_existing_transaction_hashes_to_approvers().unwrap();

            let maps_equal = hash_to_approvers_expected.iter().all(|(k , v)| hash_to_approvers_expected.get_key_value(&k).unwrap() == hash_to_approvers_observed.get_key_value(&k).unwrap());
            assert!(maps_equal);

            block_on(storage.destroy_connection());
        })
    }

    #[test]
    fn test_map_missing_transaction_hashes_to_approvers() {
        run_test(|| {
            let mut storage = SqlxBackendStorage::new();

            block_on(storage.establish_connection());

            let mut missing_hash_to_approvers_expected = storage::MissingHashesToRCApprovers::new();
            let (tx_hash, tx) =  create_random_tx();
            block_on(storage.insert_transaction(&tx_hash, &tx));
            let res = block_on(storage.find_transaction(&tx_hash));
            let found_tx = res.unwrap();
            assert_eq!(tx.nonce().0, found_tx.nonce().0);
            let mut last_tx_hash = tx_hash.clone();
            let mut all_transactions_hashes = HashSet::new();

            for i in 0 .. 2000 {
                let missing_tx_hash = rand_hash_string();
                let (tx_hash, mut tx) = match i % 3 {
                    0 => create_random_attached_tx( last_tx_hash.clone() , missing_tx_hash.clone()),
                    1 => create_random_attached_tx(missing_tx_hash.clone(), last_tx_hash.clone()),
                    2 => create_random_attached_tx(missing_tx_hash.clone(), missing_tx_hash.clone()),
                    _ => panic!("Residual is incorrect")
                };

                let mut missing_approvers  = HashSet::new();

                match i % 3 {
                    0 => missing_approvers.insert(Rc::<bundle::Hash>::new(tx_hash.clone())),
                    1 => missing_approvers.insert(Rc::<bundle::Hash>::new(tx_hash.clone())),
                    2 => missing_approvers.insert(Rc::<bundle::Hash>::new(tx_hash.clone()).clone()),
                    _ => panic!("Residual is incorrect")
                };

                missing_hash_to_approvers_expected.insert(missing_tx_hash.clone(), missing_approvers);
                block_on(storage.insert_transaction(&tx_hash, &tx));
                let res = block_on(storage.find_transaction(&tx_hash));
                let found_tx = res.unwrap();
                assert_eq!(tx.nonce().0, found_tx.nonce().0);
                all_transactions_hashes.insert(tx_hash.clone());
                last_tx_hash = tx_hash.clone();
            }

            let missing_hash_to_approvers_observed = storage.map_missing_transaction_hashes_to_approvers(all_transactions_hashes).unwrap();

            let maps_are_equal = missing_hash_to_approvers_expected.iter().all(|(k , v)| missing_hash_to_approvers_expected.get_key_value(&k).unwrap() == missing_hash_to_approvers_observed.get_key_value(&k).unwrap());


            block_on(storage.destroy_connection());
            assert!(maps_are_equal);
        })
    }

}
