    use std::time::Duration;

use pink_sidevm as sidevm;
    use log::info;
    use sidevm::{logger::Logger, ocall};

#[sidevm::main]
async fn main() {

    Logger::with_max_level(log::Level::Trace).init();

    ocall::enable_ocall_trace(true).unwrap();

    info!("starting...");

    let msg_rx = message_rx();
    tokio::select! {
        msg = msg_rx.next() => {
            info!("received msg: {:?}", msg);
            assert_eq!(msg, Some(b"foo".to_vec()));
        }
        _ = sleep::sleep(Duration::from_secs(3)) => {
            info!("slept for 3 seconds");
        },
    }
}
