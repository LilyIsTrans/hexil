/// Courtesy of https://stackoverflow.com/questions/43753491/include-git-commit-hash-as-string-into-rust-program
use std::process::Command;
fn main() {
    let head_hash = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .expect("Failed to hash HEAD!");
    let result = Command::new("git")
        .args(["status", "-s"])
        .output()
        .expect("Failed to check status!");
    let git_hash = if result.stdout.is_empty() {
        String::from_utf8(head_hash.stdout).unwrap()
    } else {
        "Unsaved Commit".to_owned()
    };
    println!("cargo:rustc-env=GIT_HASH={}", git_hash);
}
