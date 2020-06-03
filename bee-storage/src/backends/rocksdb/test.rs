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

#[cfg(test)]
mod tests {
    const BEE_TEST_DB_NAME: &str = "test_db";

    use crate::backends::rocksdb::RocksDbBackendStorage;

    use crate::tests::test::{StorageTestRunner, TestableStorage};

    impl TestableStorage for RocksDbBackendStorage {
        fn test_name() -> String {
            String::from("rocksdb")
        }

        fn setup() {}

        fn teardown() {
            rocksdb::DB::destroy(&rocksdb::Options::default(), Self::test_db_url()).unwrap();
        }

        fn test_db_url() -> String {
            String::from(BEE_TEST_DB_NAME)
        }
    }

    #[test]
    fn test_all() {
        StorageTestRunner::<RocksDbBackendStorage>::run_all_tests();
    }
}
