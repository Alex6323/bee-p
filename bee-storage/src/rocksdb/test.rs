#[cfg(test)]
mod tests {
    const BEE_TEST_DB_NAME: &str = "test_db";

    use crate::rocksdb::RocksDbBackendStorage;

    use crate::test::tests::{StorageTestRunner, TestableStorage};

    impl TestableStorage for RocksDbBackendStorage {
        fn test_name() -> String {
            String::from("rocksdb")
        }

        fn setup() -> () {}
        fn teardown() -> () {
            rocksdb::DB::destroy(&rocksdb::Options::default(), Self::test_db_url()).unwrap();
        }

        fn test_db_url() -> String {
            format!("{}", BEE_TEST_DB_NAME)
        }
    }

    #[test]
    fn test_all() {
        StorageTestRunner::<RocksDbBackendStorage>::run_all_tests();
    }
}
