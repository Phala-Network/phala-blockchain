#![allow(clippy::all)]

#![cfg(unix)]
use assert_cmd::cargo::cargo_bin;
use nix::sys::signal::Signal::{self, SIGINT, SIGTERM};
use std::{
    process::{self, Command},
    time::Duration,
};
use tempfile::tempdir;

mod common;

#[tokio::test]
async fn running_the_node_works_and_can_be_interrupted() {
    common::run_with_timeout(Duration::from_secs(60 * 10), async move {
        async fn run_command_and_kill(signal: Signal) {
            let base_path = tempdir().expect("could not create a temp dir");
            let mut cmd = common::KillChildOnDrop(
                Command::new(cargo_bin("phala-node"))
                    .stdout(process::Stdio::piped())
                    .stderr(process::Stdio::piped())
                    .args(["--dev", "-d"])
                    .arg(base_path.path())
                    .arg("--db=paritydb")
                    .arg("--no-hardware-benchmarks")
                    .spawn()
                    .unwrap(),
            );

            let stderr = cmd.stderr.take().unwrap();

            let ws_url = common::extract_info_from_output(stderr).0.ws_url;

            common::wait_n_finalized_blocks(3, &ws_url).await;

            cmd.assert_still_running();

            cmd.stop_with_signal(signal);

            // Check if the database was closed gracefully. If it was not,
            // there may exist a ref cycle that prevents the Client from being dropped properly.
            //
            // parity-db only writes the stats file on clean shutdown.
            let stats_file = base_path.path().join("chains/phala_dev/paritydb/full/stats.txt");
            assert!(std::path::Path::exists(&stats_file));
        }

        run_command_and_kill(SIGINT).await;
        run_command_and_kill(SIGTERM).await;
    })
        .await;
}

#[tokio::test]
async fn running_two_nodes_with_the_same_ws_port_should_work() {
    common::run_with_timeout(Duration::from_secs(60 * 10), async move {
        let mut first_node = common::KillChildOnDrop(common::start_node());
        let mut second_node = common::KillChildOnDrop(common::start_node());

        let stderr = first_node.stderr.take().unwrap();
        let ws_url = common::extract_info_from_output(stderr).0.ws_url;

        common::wait_n_finalized_blocks(3, &ws_url).await;

        first_node.assert_still_running();
        second_node.assert_still_running();

        first_node.stop();
        second_node.stop();
    })
        .await;
}
