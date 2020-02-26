extern crate rand;

mod sqlx;


#[cfg(test)]
mod tests {

    const BEE_TEST_DB_USER: &str = "test_db_user";
    const BEE_TEST_DB_NAME: &str = "test_db";

    use bee_bundle::{Hash, Transaction};
    use bee_storage::{Milestone, StorageBackend};
    use crate::sqlx::{SqlxBackendStorage};

    use futures::executor::block_on;
    use futures::future::{err, join_all, ok};
    use rand::Rng;
    use std::borrow::Borrow;
    use std::collections::{HashMap, HashSet};
    use std::io::{self, Write};
    use std::panic;
    use std::process::Command;
    use std::rc::Rc;
    use std::thread;
    use std::time::{Duration, Instant};

    fn test_db_url() -> String {
        format!("postgres://{}:dummy_password@localhost/{}", BEE_TEST_DB_USER, BEE_TEST_DB_NAME)
    }

    fn run_test<T>(test: T) -> ()
    where
        T: FnOnce() -> () + panic::UnwindSafe,
    {
        setup_db();

        let result = panic::catch_unwind(|| test());

        teardown_db();

        assert!(result.is_ok())
    }

    fn setup_db() -> () {
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

    fn teardown_db() -> () {
        let output = Command::new("schemes/postgress/cleanup.sh")
            .arg(BEE_TEST_DB_USER)
            .arg(BEE_TEST_DB_NAME)
            .output()
            .expect("failed to execute cleanup process");

        io::stdout().write_all(b"TEARING DOWN\n").unwrap();

        println!("status: {}", output.status);

        io::stdout().write_all(&output.stdout).unwrap();
        io::stderr().write_all(&output.stderr).unwrap();

        assert!(output.status.success());
    }

    fn test_insert_one_transaction() {
        let mut storage = SqlxBackendStorage::new();

        block_on(storage.establish_connection(test_db_url().as_str())).unwrap();
        let (tx_hash, tx) = bee_test::transaction::create_random_tx();
        block_on(storage.insert_transaction(tx_hash.clone(), tx.clone()));
        let res = block_on(storage.find_transaction(tx_hash));
        let found_tx = res.unwrap();
        block_on(storage.destroy_connection()).unwrap();
        assert_eq!(tx.nonce().0, found_tx.nonce().0);
    }

    fn test_insert_one_milestone() {
        let mut storage = SqlxBackendStorage::new();

        block_on(storage.establish_connection(test_db_url().as_str())).unwrap();
        let mut milestone = bee_test::transaction::create_random_milestone();
        milestone.index = 1;
        block_on(storage.insert_milestone(milestone.clone()));
        let res = block_on(storage.find_milestone(milestone.hash.clone()));
        let found_milestone = res.unwrap();
        block_on(storage.destroy_connection()).unwrap();
        assert_eq!(milestone.hash.to_string(), found_milestone.hash.to_string());
    }

    fn test_delete_one_transaction() {
        let mut storage = SqlxBackendStorage::new();

        block_on(storage.establish_connection(test_db_url().as_str())).unwrap();
        let (tx_hash, tx) = bee_test::transaction::create_random_tx();
        block_on(storage.insert_transaction(tx_hash.clone(), tx.clone()));
        let res = block_on(storage.find_transaction(tx_hash.clone()));
        let found_tx = res.unwrap();
        assert_eq!(tx.nonce().0, found_tx.nonce().0);
        let mut transactions_to_delete = HashSet::new();
        transactions_to_delete.insert(tx_hash);
        let res = block_on(storage.delete_transactions(&transactions_to_delete));
        assert!(res.is_ok());
        let res = block_on(
            storage.find_transaction(transactions_to_delete.iter().last().unwrap().clone()),
        );
        block_on(storage.destroy_connection()).unwrap();
        assert!(res.is_err());
    }

    fn test_delete_one_milestone() {
        let mut storage = SqlxBackendStorage::new();

        block_on(storage.establish_connection(test_db_url().as_str())).unwrap();
        let mut milestone = bee_test::transaction::create_random_milestone();
        milestone.index = 2;
        block_on(storage.insert_milestone(milestone.clone()));
        let res = block_on(storage.find_milestone(milestone.hash.clone()));
        let found_milestone = res.unwrap();
        assert_eq!(milestone.hash.to_string(), found_milestone.hash.to_string());
        let mut milestones_to_delete = HashSet::new();
        milestones_to_delete.insert(milestone.hash);
        let res = block_on(storage.delete_milestones(&milestones_to_delete));
        assert!(res.is_ok());
        let res =
            block_on(storage.find_milestone(milestones_to_delete.iter().last().unwrap().clone()));
        block_on(storage.destroy_connection()).unwrap();
        assert!(res.is_err());
    }

    fn test_transaction_multiple_delete() {
        let mut storage = SqlxBackendStorage::new();

        block_on(storage.establish_connection(test_db_url().as_str())).unwrap();

        let mut hashes = HashSet::new();

        for i in 0..10 {
            let (tx_hash, tx) = bee_test::transaction::create_random_tx();
            block_on(storage.insert_transaction(tx_hash.clone(), tx.clone()));
            let res = block_on(storage.find_transaction(tx_hash.clone()));
            let found_tx = res.unwrap();
            assert_eq!(tx.nonce().0, found_tx.nonce().0);
            hashes.insert(tx_hash);
        }

        let res = block_on(storage.delete_transactions(&hashes));
        assert!(res.is_ok());

        for hash in hashes.iter() {
            let res = block_on(storage.find_transaction(hash.clone()));
            assert!(res.is_err())
        }

        block_on(storage.destroy_connection()).unwrap();
    }

    fn test_map_hashes_to_approvers() {
        let mut storage = SqlxBackendStorage::new();

        block_on(storage.establish_connection(test_db_url().as_str())).unwrap();

        let mut hash_to_approvers_expected = bee_storage::HashesToApprovers::new();
        let (tx_hash, tx) = bee_test::transaction::create_random_tx();
        block_on(storage.insert_transaction(tx_hash.clone(), tx.clone()));
        let res = block_on(storage.find_transaction(tx_hash.clone()));
        let found_tx = res.unwrap();
        assert_eq!(tx.nonce().0, found_tx.nonce().0);
        let mut last_tx_hash = tx_hash.clone();

        for i in 0..2000 {
            let (tx_hash, mut tx) =
                bee_test::transaction::create_random_attached_tx(last_tx_hash.clone(), last_tx_hash.clone());
            let mut approvers = HashSet::new();
            approvers.insert(tx_hash.clone());
            hash_to_approvers_expected.insert(last_tx_hash.clone(), approvers);
            block_on(storage.insert_transaction(tx_hash.clone(), tx.clone()));
            let res = block_on(storage.find_transaction(tx_hash.clone()));
            let found_tx = res.unwrap();
            assert_eq!(tx.nonce().0, found_tx.nonce().0);
            last_tx_hash = tx_hash.clone();
        }

        let hash_to_approvers_observed = storage
            .map_existing_transaction_hashes_to_approvers()
            .unwrap();

        let maps_equal = hash_to_approvers_expected.iter().all(|(k, v)| {
            hash_to_approvers_expected.get_key_value(&k).unwrap()
                == hash_to_approvers_observed.get_key_value(&k).unwrap()
        });
        assert!(maps_equal);

        block_on(storage.destroy_connection()).unwrap();
    }

    fn test_map_missing_transaction_hashes_to_approvers() {
        let mut storage = SqlxBackendStorage::new();

        block_on(storage.establish_connection(test_db_url().as_str())).unwrap();

        let mut missing_hash_to_approvers_expected = bee_storage::MissingHashesToRCApprovers::new();
        let (tx_hash, tx) = bee_test::transaction::create_random_tx();
        block_on(storage.insert_transaction(tx_hash.clone(), tx.clone()));
        let res = block_on(storage.find_transaction(tx_hash.clone()));
        let found_tx = res.unwrap();
        assert_eq!(tx.nonce().0, found_tx.nonce().0);
        let mut last_tx_hash = tx_hash.clone();
        let mut all_transactions_hashes = HashSet::new();

        for i in 0..2000 {
            let missing_tx_hash_trunk = bee_test::transaction::rand_hash();
            let missing_tx_hash_branch = bee_test::transaction::rand_hash();
            let (tx_hash, mut tx) = match i % 3 {
                0 => bee_test::transaction::create_random_attached_tx(last_tx_hash.clone(), missing_tx_hash_trunk.clone()),
                1 => {
                    bee_test::transaction::create_random_attached_tx(missing_tx_hash_branch.clone(), last_tx_hash.clone())
                }
                2 => bee_test::transaction::create_random_attached_tx(
                    missing_tx_hash_branch.clone(),
                    missing_tx_hash_trunk.clone(),
                ),
                _ => panic!("Residual is incorrect"),
            };

            match i % 3 {
                0 => {
                    let mut missing_approvers = HashSet::new();
                    missing_approvers.insert(Rc::<bee_bundle::Hash>::new(tx_hash.clone()));
                    missing_hash_to_approvers_expected
                        .insert(missing_tx_hash_trunk.clone(), missing_approvers);
                }
                1 => {
                    let mut missing_approvers = HashSet::new();
                    missing_approvers.insert(Rc::<bee_bundle::Hash>::new(tx_hash.clone()));
                    missing_hash_to_approvers_expected
                        .insert(missing_tx_hash_branch.clone(), missing_approvers);
                }
                2 => {
                    let mut missing_approvers_trunk = HashSet::new();
                    let mut missing_approvers_branch = HashSet::new();
                    let rc = Rc::<bee_bundle::Hash>::new(tx_hash.clone());
                    missing_approvers_trunk.insert(rc.clone());
                    missing_approvers_branch.insert(rc.clone());
                    missing_hash_to_approvers_expected
                        .insert(missing_tx_hash_trunk.clone(), missing_approvers_trunk);
                    missing_hash_to_approvers_expected
                        .insert(missing_tx_hash_branch.clone(), missing_approvers_branch);
                }
                _ => panic!("Residual is incorrect"),
            };
            block_on(storage.insert_transaction(tx_hash.clone(), tx.clone()));
            let res = block_on(storage.find_transaction(tx_hash.clone()));
            let found_tx = res.unwrap();
            assert_eq!(tx.nonce().0, found_tx.nonce().0);
            all_transactions_hashes.insert(tx_hash.clone());
            last_tx_hash = tx_hash.clone();
        }

        let missing_hash_to_approvers_observed = storage
            .map_missing_transaction_hashes_to_approvers(all_transactions_hashes)
            .unwrap();

        let maps_are_equal = missing_hash_to_approvers_expected.iter().all(|(k, v)| {
            missing_hash_to_approvers_expected
                .get_key_value(&k)
                .unwrap()
                == missing_hash_to_approvers_observed
                    .get_key_value(&k)
                    .unwrap()
        });

        //TODO - check ref count is equal
        block_on(storage.destroy_connection()).unwrap();
        assert!(maps_are_equal);
    }

    fn test_insert_transactions_concurrent() {
        let mut storage = SqlxBackendStorage::new();
        block_on(storage.establish_connection(test_db_url().as_str())).unwrap();
        let mut hashes_transaction_seq = Vec::new();
        let mut hashes = HashSet::new();
        const NUM_TRANSACTIONS: usize = 4000;

        let mut futures = Vec::new();
        for i in 0..NUM_TRANSACTIONS {
            let (tx_hash, tx) = bee_test::transaction::create_random_tx();
            hashes.insert(tx_hash.clone());
            hashes_transaction_seq.push((tx_hash, tx));
        }

        let now = Instant::now();
        for (hash, tx) in hashes_transaction_seq {
            let f = storage.insert_transaction(hash, tx);
            futures.push(f);
        }

        block_on(join_all(futures));
        let message = format!(
            "\ntest_insert_transactions_concurrent Elapsed: {}\n",
            now.elapsed().as_secs()
        );
        io::stdout().write_all(message.as_bytes()).unwrap();
        for h in hashes {
            let res = block_on(storage.find_transaction(h));
            assert!(res.is_ok());
        }
        block_on(storage.destroy_connection()).unwrap();
    }

    fn test_insert_transactions_batch() {
        let mut storage = SqlxBackendStorage::new();
        block_on(storage.establish_connection(test_db_url().as_str())).unwrap();
        let mut hashes_to_transactions = HashMap::new();
        let mut hashes = HashSet::new();
        const NUM_TRANSACTIONS: usize = 10000;

        for i in 0..NUM_TRANSACTIONS {
            let (tx_hash, tx) = bee_test::transaction::create_random_tx();
            hashes.insert(tx_hash.clone());
            hashes_to_transactions.insert(tx_hash, tx);
        }

        let now = Instant::now();
        block_on(storage.insert_transactions(hashes_to_transactions));
        let message = format!(
            "\ntest_insert_transactions_batch Elapsed: {}\n",
            now.elapsed().as_secs()
        );
        io::stdout().write_all(message.as_bytes()).unwrap();

        for h in hashes {
            let res = block_on(storage.find_transaction(h));
            assert!(res.is_ok());
        }

        block_on(storage.destroy_connection()).unwrap();
    }

    #[test]
    fn test_all() {
        run_test(|| {
            test_insert_one_transaction();
            test_insert_one_milestone();
            test_delete_one_transaction();
            test_delete_one_milestone();
            test_transaction_multiple_delete();
            test_map_hashes_to_approvers();
            test_map_missing_transaction_hashes_to_approvers();
            test_insert_transactions_concurrent();
            test_insert_transactions_batch();
        })
    }
}
