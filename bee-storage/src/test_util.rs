use std::panic;

pub trait StorageTest {
    fn setup();
    fn teardown();
    fn test_insert_one_transaction();
    fn test_insert_one_milestone();
    fn test_delete_one_transaction();
    fn test_delete_one_milestone();
    fn test_transaction_multiple_delete();
    fn test_map_hashes_to_approvers();
    fn test_map_missing_transaction_hashes_to_approvers();
    fn test_insert_transactions_concurrent();
    fn test_insert_transactions_batch();
}

pub struct StorageTestRunner {}

impl StorageTestRunner {
    pub fn run_test<T, U>(test: T) -> ()
    where
        T: FnOnce() -> () + panic::UnwindSafe,
        U: StorageTest,
    {
        U::setup();

        let result = panic::catch_unwind(|| test());

        U::teardown();

        assert!(result.is_ok())
    }
}
