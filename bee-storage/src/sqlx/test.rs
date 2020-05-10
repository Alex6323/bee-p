#[cfg(test)]
mod tests {
    const BEE_TEST_DB_USER: &str = "test_db_user";
    const BEE_TEST_DB_NAME: &str = "test_db";

    use crate::sqlx::SqlxBackendStorage;

    use std::{
        io::{self, Write},
        process::Command,
    };

    use crate::test::tests::{StorageTestRunner, TestableStorage};

    impl TestableStorage for SqlxBackendStorage {
        fn test_name() -> String {
            String::from("sqlx")
        }

        fn setup() -> () {
            let output = Command::new("src/sqlx/schemes/postgres/setup.sh")
                .arg("src/sqlx/schemes/postgres/schema.sql")
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

        fn teardown() -> () {
            let output = Command::new("src/sqlx/schemes/postgres/cleanup.sh")
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

        fn test_db_url() -> String {
            format!(
                "postgres://{}:dummy_password@localhost/{}",
                BEE_TEST_DB_USER, BEE_TEST_DB_NAME
            )
        }
    }

    #[test]
    fn test_all() {
        StorageTestRunner::<SqlxBackendStorage>::run_all_tests();
    }
}
