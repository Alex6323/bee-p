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

use crate::{backends::sqlx::SqlxBackendStorage, tests::test::TestableStorage};

use std::{
    io::{self, Write},
    process::Command,
};

const BEE_TEST_DB_USER: &str = "test_db_user";
const BEE_TEST_DB_NAME: &str = "test_db";

impl TestableStorage for SqlxBackendStorage {
    fn test_name() -> String {
        String::from("sqlx")
    }

    fn setup() {
        let output = Command::new("src/backends/sqlx/schemes/postgres/setup.sh")
            .arg("src/backends/sqlx/schemes/postgres/schema.sql")
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

    fn teardown() {
        let output = Command::new("src/backends/sqlx/schemes/postgres/teardown.sh")
            .arg(BEE_TEST_DB_USER)
            .arg(BEE_TEST_DB_NAME)
            .output()
            .expect("failed to execute teardown process");

        io::stdout().write_all(b"TEARING DOWN\n").unwrap();

        println!("status: {}", output.status);

        io::stdout().write_all(&output.stdout).unwrap();
        io::stderr().write_all(&output.stderr).unwrap();

        assert!(output.status.success());
    }

    fn test_db_url() -> String {
        format!(
            "postgres://{}:dummy_password@localhost/{}",
            BEE_TEST_DB_USER, BEE_TEST_DB_NAME
        )
    }
}

#[cfg(test)]
mod tests {

    use crate::{backends::sqlx::SqlxBackendStorage, tests::test::StorageTestRunner};

    #[test]
    fn test_all() {
        StorageTestRunner::<SqlxBackendStorage>::run_all_tests();
    }
}
