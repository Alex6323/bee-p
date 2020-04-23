use std::process::Command;

#[derive(Debug)]
enum BuildError {
    GitCommit,
}

fn main() -> Result<(), BuildError> {
    match Command::new("git").args(&["rev-parse", "HEAD"]).output() {
        Ok(output) => {
            println!(
                "cargo:rustc-env=BEE_GIT_COMMIT={}",
                String::from_utf8(output.stdout).unwrap()
            );
            Ok(())
        }
        Err(_) => return Err(BuildError::GitCommit),
    }
}
