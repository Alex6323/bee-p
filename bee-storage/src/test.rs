use crate::storage::StorageBackend;

use futures::{
    executor::block_on,
    future::join_all,
};

use std::{
    collections::{
        HashMap,
        HashSet,
    },
    io::{
        self,
        Write,
    },
    marker::PhantomData,
    panic,
    rc::Rc,
    time::Instant,
};

use bee_bundle::{
    Address,
    Hash,
    Payload,
    TransactionField,
};

use crate::storage::{
    HashesToApprovers,
    MissingHashesToRCApprovers,
    StateDeltaMap,
};

pub trait TestableStorage {
    fn setup() -> ();
    fn teardown() -> ();
    fn test_db_url() -> String;
}
pub struct StorageTestRunner<T: TestableStorage + StorageBackend> {
    phantom: PhantomData<T>,
}

impl<T: TestableStorage + StorageBackend> StorageTestRunner<T> {
    fn test_insert_one_transaction() {
        let mut storage = T::new();

        block_on(storage.establish_connection(T::test_db_url().as_str())).unwrap();
        let (tx_hash, tx) = bee_test::transaction::create_random_tx();
        block_on(storage.insert_transaction(tx_hash.clone(), bee_test::transaction::clone_tx(&tx))).unwrap();
        let res = block_on(storage.find_transaction(tx_hash));
        let found_tx = res.unwrap();
        block_on(storage.destroy_connection()).unwrap();

        assert_eq!(tx.payload().to_inner().len(), found_tx.payload().to_inner().len());
        assert_eq!(tx.bundle(), found_tx.bundle());
        assert_eq!(tx.trunk(), found_tx.trunk());
        assert_eq!(tx.branch(), found_tx.branch());
        assert_eq!(tx.tag(), found_tx.tag());
        assert_eq!(tx.obsolete_tag(), found_tx.obsolete_tag());
        assert_eq!(tx.nonce(), found_tx.nonce());
    }

    fn test_insert_one_milestone() {
        let mut storage = T::new();

        block_on(storage.establish_connection(T::test_db_url().as_str())).unwrap();
        let mut milestone = bee_test::transaction::create_random_milestone();
        milestone.index = 1;
        block_on(storage.insert_milestone(milestone.clone())).unwrap();
        let res = block_on(storage.find_milestone(milestone.hash.clone()));
        let found_milestone = res.unwrap();
        block_on(storage.destroy_connection()).unwrap();

        assert_eq!(milestone.hash, found_milestone.hash);
    }

    fn test_delete_one_transaction() {
        let mut storage = T::new();

        block_on(storage.establish_connection(T::test_db_url().as_str())).unwrap();
        let (tx_hash, tx) = bee_test::transaction::create_random_tx();
        block_on(storage.insert_transaction(tx_hash.clone(), bee_test::transaction::clone_tx(&tx))).unwrap();
        let res = block_on(storage.find_transaction(tx_hash.clone()));
        let found_tx = res.unwrap();
        assert_eq!(tx.nonce(), found_tx.nonce());
        let mut transactions_to_delete = HashSet::new();
        transactions_to_delete.insert(tx_hash);
        let res = block_on(storage.delete_transactions(&transactions_to_delete));
        assert!(res.is_ok());
        let res = block_on(storage.find_transaction(transactions_to_delete.iter().last().unwrap().clone()));
        block_on(storage.destroy_connection()).unwrap();
        assert!(res.is_err());
    }

    fn test_delete_one_milestone() {
        let mut storage = T::new();

        block_on(storage.establish_connection(T::test_db_url().as_str())).unwrap();
        let mut milestone = bee_test::transaction::create_random_milestone();
        milestone.index = 2;
        block_on(storage.insert_milestone(milestone.clone())).unwrap();
        let res = block_on(storage.find_milestone(milestone.hash.clone()));
        let found_milestone = res.unwrap();
        assert_eq!(milestone.hash, found_milestone.hash);
        let mut milestones_to_delete = HashSet::new();
        milestones_to_delete.insert(milestone.hash);
        let res = block_on(storage.delete_milestones(&milestones_to_delete));
        assert!(res.is_ok());
        let res = block_on(storage.find_milestone(milestones_to_delete.iter().last().unwrap().to_owned()));
        block_on(storage.destroy_connection()).unwrap();
        assert!(res.is_err());
    }

    fn test_transaction_multiple_delete() {
        let mut storage = T::new();

        block_on(storage.establish_connection(T::test_db_url().as_str())).unwrap();

        let mut hashes = HashSet::new();

        for _i in 0..10 {
            let (tx_hash, tx) = bee_test::transaction::create_random_tx();
            block_on(storage.insert_transaction(tx_hash.clone(), bee_test::transaction::clone_tx(&tx))).unwrap();
            let res = block_on(storage.find_transaction(tx_hash.clone()));
            let found_tx = res.unwrap();
            assert_eq!(tx.nonce(), found_tx.nonce());
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
        let mut storage = T::new();

        block_on(storage.establish_connection(T::test_db_url().as_str())).unwrap();

        let mut hash_to_approvers_expected = HashesToApprovers::new();
        let (tx_hash, tx) = bee_test::transaction::create_random_tx();
        block_on(storage.insert_transaction(tx_hash.clone(), bee_test::transaction::clone_tx(&tx))).unwrap();
        let res = block_on(storage.find_transaction(tx_hash.clone()));
        let found_tx = res.unwrap();
        assert_eq!(tx.nonce(), found_tx.nonce());
        let mut last_tx_hash = tx_hash.clone();

        for _i in 0..2000 {
            let (tx_hash, tx) =
                bee_test::transaction::create_random_attached_tx(last_tx_hash.clone(), last_tx_hash.clone());
            let mut approvers = HashSet::new();
            approvers.insert(tx_hash.clone());
            hash_to_approvers_expected.insert(last_tx_hash.clone(), approvers);
            block_on(storage.insert_transaction(tx_hash.clone(), bee_test::transaction::clone_tx(&tx))).unwrap();
            let res = block_on(storage.find_transaction(tx_hash.clone()));
            let found_tx = res.unwrap();
            assert_eq!(tx.nonce(), found_tx.nonce());
            last_tx_hash = tx_hash.clone();
        }

        let hash_to_approvers_observed = storage.map_existing_transaction_hashes_to_approvers().unwrap();

        let maps_equal = hash_to_approvers_expected.iter().all(|(k, _v)| {
            hash_to_approvers_expected.get_key_value(&k).unwrap()
                == hash_to_approvers_observed.get_key_value(&k).unwrap()
        });
        assert!(maps_equal);

        block_on(storage.destroy_connection()).unwrap();
    }

    fn test_map_missing_transaction_hashes_to_approvers() {
        let mut storage = T::new();

        block_on(storage.establish_connection(T::test_db_url().as_str())).unwrap();

        let mut missing_hash_to_approvers_expected = MissingHashesToRCApprovers::new();
        let (tx_hash, tx) = bee_test::transaction::create_random_tx();
        block_on(storage.insert_transaction(tx_hash.clone(), bee_test::transaction::clone_tx(&tx))).unwrap();
        let res = block_on(storage.find_transaction(tx_hash.clone()));
        let found_tx = res.unwrap();
        assert_eq!(tx.nonce(), found_tx.nonce());
        let mut last_tx_hash = tx_hash.clone();
        let mut all_transactions_hashes = HashSet::new();

        for i in 0..2000 {
            let missing_tx_hash_trunk = bee_test::transaction::rand_trits_field::<Hash>();
            let missing_tx_hash_branch = bee_test::transaction::rand_trits_field::<Hash>();
            let (tx_hash, tx) = match i % 3 {
                0 => bee_test::transaction::create_random_attached_tx(
                    last_tx_hash.clone(),
                    missing_tx_hash_trunk.clone(),
                ),
                1 => bee_test::transaction::create_random_attached_tx(
                    missing_tx_hash_branch.clone(),
                    last_tx_hash.clone(),
                ),
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
                    missing_hash_to_approvers_expected.insert(missing_tx_hash_trunk.clone(), missing_approvers);
                }
                1 => {
                    let mut missing_approvers = HashSet::new();
                    missing_approvers.insert(Rc::<bee_bundle::Hash>::new(tx_hash.clone()));
                    missing_hash_to_approvers_expected.insert(missing_tx_hash_branch.clone(), missing_approvers);
                }
                2 => {
                    let mut missing_approvers_trunk = HashSet::new();
                    let mut missing_approvers_branch = HashSet::new();
                    let rc = Rc::<bee_bundle::Hash>::new(tx_hash.clone());
                    missing_approvers_trunk.insert(rc.clone());
                    missing_approvers_branch.insert(rc.clone());
                    missing_hash_to_approvers_expected.insert(missing_tx_hash_trunk.clone(), missing_approvers_trunk);
                    missing_hash_to_approvers_expected.insert(missing_tx_hash_branch.clone(), missing_approvers_branch);
                }
                _ => panic!("Residual is incorrect"),
            };
            block_on(storage.insert_transaction(tx_hash.clone(), bee_test::transaction::clone_tx(&tx))).unwrap();
            let res = block_on(storage.find_transaction(tx_hash.clone()));
            let found_tx = res.unwrap();
            assert_eq!(tx.nonce(), found_tx.nonce());
            all_transactions_hashes.insert(tx_hash.clone());
            last_tx_hash = tx_hash.clone();
        }

        let missing_hash_to_approvers_observed = storage
            .map_missing_transaction_hashes_to_approvers(all_transactions_hashes)
            .unwrap();

        let maps_are_equal = missing_hash_to_approvers_expected.iter().all(|(k, _v)| {
            missing_hash_to_approvers_expected.get_key_value(&k).unwrap()
                == missing_hash_to_approvers_observed.get_key_value(&k).unwrap()
        });

        //TODO - check ref count is equal
        block_on(storage.destroy_connection()).unwrap();
        assert!(maps_are_equal);
    }

    fn test_insert_transactions_concurrent() {
        let mut storage = T::new();
        block_on(storage.establish_connection(T::test_db_url().as_str())).unwrap();
        let mut hashes_transaction_seq = Vec::new();
        let mut hashes = HashSet::new();
        const NUM_TRANSACTIONS: usize = 1000;

        let mut futures = Vec::new();
        for _i in 0..NUM_TRANSACTIONS {
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
        let mut storage = T::new();
        block_on(storage.establish_connection(T::test_db_url().as_str())).unwrap();
        let mut hashes_to_transactions = HashMap::new();
        let mut hashes = HashSet::new();
        const NUM_TRANSACTIONS: usize = 1000;

        for _i in 0..NUM_TRANSACTIONS {
            let (tx_hash, tx) = bee_test::transaction::create_random_tx();
            hashes.insert(tx_hash.clone());
            hashes_to_transactions.insert(tx_hash, tx);
        }

        let now = Instant::now();
        block_on(storage.insert_transactions(hashes_to_transactions)).unwrap();
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
}

impl<T: TestableStorage + StorageBackend> StorageTestRunner<T> {
    pub fn run_test<F>(test: F) -> ()
    where
        F: FnOnce() -> () + panic::UnwindSafe,
    {
        T::setup();

        let result = panic::catch_unwind(|| test());

        T::teardown();

        assert!(result.is_ok())
    }

    pub fn run_all_tests() {
        StorageTestRunner::<T>::run_test(|| {
            Self::test_insert_one_transaction();
            Self::test_insert_one_milestone();
            Self::test_delete_one_transaction();
            Self::test_delete_one_milestone();
            Self::test_transaction_multiple_delete();
            Self::test_map_hashes_to_approvers();
            Self::test_map_missing_transaction_hashes_to_approvers();
            Self::test_insert_transactions_concurrent();
            Self::test_insert_transactions_batch();
        })
    }
}