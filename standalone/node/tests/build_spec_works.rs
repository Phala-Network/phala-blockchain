#![allow(clippy::all)]

use assert_cmd::cargo::cargo_bin;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn build_spec_works() {
    let base_path = tempdir().expect("could not create a temp dir");

    let output = Command::new(cargo_bin("phala-node"))
        .args(["build-spec", "--dev", "-d"])
        .arg(base_path.path())
        .output()
        .unwrap();
    assert!(output.status.success());

    // Make sure that the `dev` chain folder exists, but the `db` doesn't
    assert!(base_path.path().join("chains/phala_dev/").exists());
    assert!(!base_path.path().join("chains/phala_dev/db").exists());

    let _value: serde_json::Value = serde_json::from_slice(output.stdout.as_slice()).unwrap();
}
