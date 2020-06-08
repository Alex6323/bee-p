// Copyright 2020 IOTA Stiftung
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with
// the License. You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on
// an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and limitations under the License.

use crate::storage::StorageBackend;

use std::marker::PhantomData;

pub trait TestableStorage {
    fn test_name() -> String;
    fn setup();
    fn teardown();
    fn test_db_url() -> String;
}

pub struct StorageTestRunner<T: TestableStorage + StorageBackend> {
    phantom: PhantomData<T>,
}

#[cfg(test)]
pub mod tests {
    use crate::storage::{HashesToApprovers, MissingHashesToRCApprovers, StateDeltaMap, StorageBackend};

    use bee_test::field::rand_trits_field;
    use bee_transaction::{Address, Hash};

    use std::{
        collections::{HashMap, HashSet},
        io::{self, Write},
        panic,
        rc::Rc,
        time::Instant,
    };

    use futures::{executor::block_on, future::join_all};

    use super::{StorageTestRunner, TestableStorage};

    impl<T: TestableStorage + StorageBackend> StorageTestRunner<T> {
        fn test_insert_one_transaction() {
            let mut storage = T::new();

            block_on(storage.establish_connection(T::test_db_url().as_str())).unwrap();
            let (tx_hash, tx) = bee_test::transaction::create_random_tx();
            block_on(storage.insert_transaction(tx_hash, bee_test::transaction::clone_tx(&tx))).unwrap();
            let res = block_on(storage.find_transaction(tx_hash));
            let found_tx = res.unwrap();

            block_on(storage.destroy_connection()).unwrap();

            assert_eq!(tx, found_tx);
        }

        fn test_transaction_update_solid() {
            let mut storage = T::new();

            block_on(storage.establish_connection(T::test_db_url().as_str())).unwrap();
            let (tx_hash, tx) = bee_test::transaction::create_random_tx();
            block_on(storage.insert_transaction(tx_hash, bee_test::transaction::clone_tx(&tx))).unwrap();
            let res = block_on(storage.find_transaction(tx_hash));
            let found_tx = res.unwrap();
            assert_eq!(tx, found_tx);
            let solid_state_res = block_on(storage.get_transactions_solid_state(vec![tx_hash]));
            assert_eq!(solid_state_res.unwrap()[0], false);
            block_on(storage.update_transactions_set_solid(vec![tx_hash].into_iter().collect())).unwrap();

            let solid_state_res = block_on(storage.get_transactions_solid_state(vec![tx_hash]));
            assert_eq!(solid_state_res.unwrap()[0], true);
            block_on(storage.destroy_connection()).unwrap();
        }

        fn test_transaction_snapshot_index() {
            let mut storage = T::new();

            block_on(storage.establish_connection(T::test_db_url().as_str())).unwrap();
            let (tx_hash, tx) = bee_test::transaction::create_random_tx();
            block_on(storage.insert_transaction(tx_hash, bee_test::transaction::clone_tx(&tx))).unwrap();
            let res = block_on(storage.find_transaction(tx_hash));
            let found_tx = res.unwrap();
            assert_eq!(tx, found_tx);
            let snapshot_index_res = block_on(storage.get_transactions_snapshot_index(vec![tx_hash]));
            assert_eq!(snapshot_index_res.unwrap()[0], 0);
            block_on(storage.update_transactions_set_snapshot_index(vec![tx_hash].into_iter().collect(), 1)).unwrap();

            let snapshot_index_res = block_on(storage.get_transactions_snapshot_index(vec![tx_hash]));
            assert_eq!(snapshot_index_res.unwrap()[0], 1);
            block_on(storage.destroy_connection()).unwrap();
        }

        fn test_insert_one_milestone() {
            let mut storage = T::new();

            block_on(storage.establish_connection(T::test_db_url().as_str())).unwrap();
            let milestone = bee_test::milestone::create_random_milestone(1);
            block_on(storage.insert_milestone(bee_test::milestone::clone_ms(&milestone))).unwrap();
            let res = block_on(storage.find_milestone(*milestone.hash()));
            let found_milestone = res.unwrap();
            block_on(storage.destroy_connection()).unwrap();

            assert_eq!(milestone.hash(), found_milestone.hash());
        }

        fn test_delete_one_transaction() {
            let mut storage = T::new();

            block_on(storage.establish_connection(T::test_db_url().as_str())).unwrap();
            let (tx_hash, tx) = bee_test::transaction::create_random_tx();
            block_on(storage.insert_transaction(tx_hash, bee_test::transaction::clone_tx(&tx))).unwrap();
            let res = block_on(storage.find_transaction(tx_hash));
            let found_tx = res.unwrap();
            assert_eq!(tx, found_tx);
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
            let milestone = bee_test::milestone::create_random_milestone(2);
            block_on(storage.insert_milestone(bee_test::milestone::clone_ms(&milestone))).unwrap();
            let res = block_on(storage.find_milestone(*milestone.hash()));
            let found_milestone = res.unwrap();
            assert_eq!(milestone.hash(), found_milestone.hash());
            let mut milestones_to_delete = HashSet::new();
            // TODO Delete by index ?
            milestones_to_delete.insert(*milestone.hash());
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
                block_on(storage.insert_transaction(tx_hash, bee_test::transaction::clone_tx(&tx))).unwrap();
                let res = block_on(storage.find_transaction(tx_hash));
                let found_tx = res.unwrap();
                assert_eq!(tx, found_tx);
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
            block_on(storage.insert_transaction(tx_hash, bee_test::transaction::clone_tx(&tx))).unwrap();
            let res = block_on(storage.find_transaction(tx_hash));
            let found_tx = res.unwrap();
            assert_eq!(tx, found_tx);
            let mut last_tx_hash = tx_hash;

            for _i in 0..1200 {
                let (tx_hash, tx) = bee_test::transaction::create_random_attached_tx(last_tx_hash, last_tx_hash);
                let mut approvers = HashSet::new();
                approvers.insert(tx_hash);
                hash_to_approvers_expected.insert(last_tx_hash, approvers);
                block_on(storage.insert_transaction(tx_hash, bee_test::transaction::clone_tx(&tx))).unwrap();
                let res = block_on(storage.find_transaction(tx_hash));
                let found_tx = res.unwrap();
                assert_eq!(tx, found_tx);
                last_tx_hash = tx_hash;
            }

            let now = Instant::now();
            let hash_to_approvers_observed = storage.map_existing_transaction_hashes_to_approvers().unwrap();
            let message = format!(
                "\n{}: test_map_hashes_to_approvers milliseconds elapsed: {}\n",
                T::test_name(),
                now.elapsed().as_millis()
            );
            io::stdout().write_all(message.as_bytes()).unwrap();

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
            block_on(storage.insert_transaction(tx_hash, bee_test::transaction::clone_tx(&tx))).unwrap();
            let res = block_on(storage.find_transaction(tx_hash));
            let found_tx = res.unwrap();
            assert_eq!(tx, found_tx);
            let mut last_tx_hash = tx_hash;
            let mut all_transactions_hashes = HashSet::new();

            for i in 0..1200 {
                let missing_tx_hash_trunk = rand_trits_field::<Hash>();
                let missing_tx_hash_branch = rand_trits_field::<Hash>();
                let (tx_hash, tx) = match i % 3 {
                    0 => bee_test::transaction::create_random_attached_tx(last_tx_hash, missing_tx_hash_trunk),
                    1 => bee_test::transaction::create_random_attached_tx(missing_tx_hash_branch, last_tx_hash),
                    2 => {
                        bee_test::transaction::create_random_attached_tx(missing_tx_hash_branch, missing_tx_hash_trunk)
                    }
                    _ => panic!("Residual is incorrect"),
                };

                match i % 3 {
                    0 => {
                        let mut missing_approvers = HashSet::new();
                        missing_approvers.insert(Rc::<Hash>::new(tx_hash));
                        missing_hash_to_approvers_expected.insert(missing_tx_hash_trunk, missing_approvers);
                    }
                    1 => {
                        let mut missing_approvers = HashSet::new();
                        missing_approvers.insert(Rc::<Hash>::new(tx_hash));
                        missing_hash_to_approvers_expected.insert(missing_tx_hash_branch, missing_approvers);
                    }
                    2 => {
                        let mut missing_approvers_trunk = HashSet::new();
                        let mut missing_approvers_branch = HashSet::new();
                        let rc = Rc::<Hash>::new(tx_hash);
                        missing_approvers_trunk.insert(rc.clone());
                        missing_approvers_branch.insert(rc.clone());
                        missing_hash_to_approvers_expected.insert(missing_tx_hash_trunk, missing_approvers_trunk);
                        missing_hash_to_approvers_expected.insert(missing_tx_hash_branch, missing_approvers_branch);
                    }
                    _ => panic!("Residual is incorrect"),
                };
                block_on(storage.insert_transaction(tx_hash, bee_test::transaction::clone_tx(&tx))).unwrap();
                let res = block_on(storage.find_transaction(tx_hash));
                let found_tx = res.unwrap();
                assert_eq!(tx, found_tx);
                all_transactions_hashes.insert(tx_hash);
                last_tx_hash = tx_hash;
            }

            let now = Instant::now();
            let missing_hash_to_approvers_observed = storage
                .map_missing_transaction_hashes_to_approvers(all_transactions_hashes)
                .unwrap();
            let message = format!(
                "\n{}: test_map_missing_transaction_hashes_to_approvers milliseconds elapsed: {}\n",
                T::test_name(),
                now.elapsed().as_millis()
            );
            io::stdout().write_all(message.as_bytes()).unwrap();

            let maps_are_equal = missing_hash_to_approvers_expected.iter().all(|(k, _v)| {
                missing_hash_to_approvers_expected.get_key_value(&k).unwrap()
                    == missing_hash_to_approvers_observed.get_key_value(&k).unwrap()
            });

            // TODO - check ref count is equal
            block_on(storage.destroy_connection()).unwrap();
            assert!(maps_are_equal);
        }

        fn test_insert_transactions_concurrent() {
            let mut storage = T::new();
            block_on(storage.establish_connection(T::test_db_url().as_str())).unwrap();
            let mut hashes_transaction_seq = Vec::new();
            let mut hashes = HashSet::new();
            const NUM_TRANSACTIONS: usize = 2000;

            let mut futures = Vec::new();
            for _i in 0..NUM_TRANSACTIONS {
                let (tx_hash, tx) = bee_test::transaction::create_random_tx();
                hashes.insert(tx_hash);
                hashes_transaction_seq.push((tx_hash, tx));
            }

            let now = Instant::now();
            for (hash, tx) in hashes_transaction_seq {
                let f = storage.insert_transaction(hash, tx);
                futures.push(f);
            }
            block_on(join_all(futures));
            let message = format!(
                "\n{}: test_insert_transactions_concurrent milliseconds elapsed: {}\n",
                T::test_name(),
                now.elapsed().as_millis()
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
            const NUM_TRANSACTIONS: usize = 2048;

            for _i in 0..NUM_TRANSACTIONS {
                let (tx_hash, tx) = bee_test::transaction::create_random_tx();
                hashes.insert(tx_hash);
                hashes_to_transactions.insert(tx_hash, tx);
            }

            let now = Instant::now();
            block_on(storage.insert_transactions(hashes_to_transactions)).unwrap();
            let message = format!(
                "\n{}: test_insert_transactions_batch milliseconds elapsed (insert operation): {}\n",
                T::test_name(),
                now.elapsed().as_millis()
            );
            io::stdout().write_all(message.as_bytes()).unwrap();

            let now = Instant::now();
            for h in hashes {
                let res = block_on(storage.find_transaction(h));
                assert!(res.is_ok());
            }
            let message = format!(
                "\n{}: test_insert_transactions_batch milliseconds elapsed (find operation): {}\n",
                T::test_name(),
                now.elapsed().as_millis()
            );
            io::stdout().write_all(message.as_bytes()).unwrap();

            block_on(storage.destroy_connection()).unwrap();
        }

        fn test_insert_transactions_batch_concurrent() {
            let mut storage = T::new();
            block_on(storage.establish_connection(T::test_db_url().as_str())).unwrap();
            let mut batches = Vec::new();
            let mut hashes = HashSet::new();
            const NUM_TRANSACTIONS: usize = 2048;

            for _i in 0..num_cpus::get() {
                let mut hashes_to_transactions_batch = HashMap::new();
                for _i in 0..NUM_TRANSACTIONS / num_cpus::get() {
                    let (tx_hash, tx) = bee_test::transaction::create_random_tx();
                    hashes.insert(tx_hash);
                    hashes_to_transactions_batch.insert(tx_hash, tx);
                }
                batches.push(hashes_to_transactions_batch);
            }

            let mut futures = Vec::new();
            let now = Instant::now();
            for b in batches {
                let f = storage.insert_transactions(b);
                futures.push(f);
            }

            block_on(join_all(futures));
            let message = format!(
                "\n{}: test_insert_transactions_batch_concurrent milliseconds elapsed: {}\n",
                T::test_name(),
                now.elapsed().as_millis()
            );
            io::stdout().write_all(message.as_bytes()).unwrap();

            for h in hashes {
                let res = block_on(storage.find_transaction(h));
                assert!(res.is_ok());
            }

            block_on(storage.destroy_connection()).unwrap();
        }

        fn test_store_and_load_state_delta() {
            let mut storage = T::new();
            block_on(storage.establish_connection(T::test_db_url().as_str())).unwrap();

            let milestone = bee_test::milestone::create_random_milestone(100_000);
            block_on(storage.insert_milestone(bee_test::milestone::clone_ms(&milestone))).unwrap();
            block_on(storage.find_milestone(*milestone.hash())).unwrap();

            let mut state_delta = StateDeltaMap {
                address_to_delta: HashMap::new(),
            };
            let mut addresses = HashSet::new();
            const NUM_BALANCES: usize = 1000;
            for _i in 0..NUM_BALANCES {
                let address = rand_trits_field::<Address>();
                addresses.insert(address.clone());
                state_delta.address_to_delta.insert(address.clone(), _i as i64);
            }

            for i in 0..NUM_BALANCES {
                let address = rand_trits_field::<Address>();
                addresses.insert(address.clone());
                state_delta.address_to_delta.insert(address.clone(), -(i as i64));
            }

            let now = Instant::now();
            block_on(storage.insert_state_delta(state_delta, milestone.index())).unwrap();
            let message = format!(
                "\n{}: test_store_and_load_state_delta milliseconds elapsed: {}\n",
                T::test_name(),
                now.elapsed().as_millis()
            );
            io::stdout().write_all(message.as_bytes()).unwrap();

            let res = block_on(storage.load_state_delta(milestone.index()));
            assert!(res.is_ok());

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
                Self::test_delete_one_transaction();
                Self::test_transaction_multiple_delete();
                Self::test_map_hashes_to_approvers();
                Self::test_map_missing_transaction_hashes_to_approvers();
                Self::test_insert_one_milestone();
                Self::test_delete_one_milestone();
                Self::test_insert_transactions_concurrent();
                Self::test_insert_transactions_batch();
                Self::test_insert_transactions_batch_concurrent();
                Self::test_store_and_load_state_delta();
                Self::test_transaction_update_solid();
                Self::test_transaction_snapshot_index();
            })
        }
    }
}
