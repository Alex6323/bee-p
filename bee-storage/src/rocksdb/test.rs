#[cfg(test)]
mod tests {
    const BEE_TEST_DB_NAME: &str = "test_db";

    use crate::rocksdb::RocksDbBackendStorage;

    use std::{
        io::{
            self,
            Write,
        },
        process::Command,
    };

    use crate::test::{
        StorageTestRunner,
        TestableStorage,
    };

    impl TestableStorage for RocksDbBackendStorage {
        fn setup() -> () {}

        fn teardown() -> () {
            rocksdb::DB::destroy(&rocksdb::Options::default(), Self::test_db_url());
        }
        fn test_db_url() -> String {
            format!("{}", BEE_TEST_DB_NAME)
        }

        fn test_name() -> String {
            String::from("rocksdb")
        }
    }

    #[test]
    fn test_all() {
        StorageTestRunner::<RocksDbBackendStorage>::run_all_tests();
    }
}
