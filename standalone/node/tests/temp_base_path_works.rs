#![allow(clippy::all)]

#![cfg(unix)]

use assert_cmd::cargo::cargo_bin;
use std::{
    process::{Command, Stdio},
    time::Duration,
};

mod common;

#[allow(dead_code)]
// Apparently `#[ignore]` doesn't actually work to disable this one.
//#[tokio::test]
async fn temp_base_path_works() {
    common::run_with_timeout(Duration::from_secs(60 * 10), async move {
        let mut cmd = Command::new(cargo_bin("phala-node"));
        let mut child = common::KillChildOnDrop(
            cmd.args(["--dev", "--tmp", "--no-hardware-benchmarks"])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap(),
        );

        let mut stderr = child.stderr.take().unwrap();
        let node_info = common::extract_info_from_output(&mut stderr).0;

        // Let it produce some blocks.
        common::wait_n_finalized_blocks(3, &node_info.ws_url).await;

        // Ensure the db path exists while the node is running
        assert!(node_info.db_path.exists());

        child.assert_still_running();

        // Stop the process
        child.stop();

        if node_info.db_path.exists() {
            panic!("Database path `{}` wasn't deleted!", node_info.db_path.display());
        }
    })
        .await;
}
