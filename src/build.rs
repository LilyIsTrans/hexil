/// Courtesy of https://stackoverflow.com/questions/43753491/include-git-commit-hash-as-string-into-rust-program
use std::process::Command;
fn main() {
    let head_hash = Command::new("git")
        .args(&["rev-parse", "HEAD"])
        .output()
        .expect("Failed to hash HEAD");
    let result = Command::new("git")
        .args(&["rev-parse", head_hash])
        .status()
        .expect("Failed to check for commit hash!");
    let git_hash = String::from_utf8(output.stdout).unwrap();
    println!("cargo:rustc-env=GIT_HASH={}", git_hash);
}
