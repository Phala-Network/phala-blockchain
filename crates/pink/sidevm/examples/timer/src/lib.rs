use futures::stream::futures_unordered::FuturesUnordered;
use log::info;

use pink_sidevm as sidevm;
use sidevm::{logger::Logger, ocall};

#[sidevm::main]
async fn main() {
    Logger::with_max_level(log::Level::Trace).init();
    ocall::enable_ocall_trace(true).unwrap();
    info!("starting...");
    for _ in 0..20 {
        sidevm::spawn(async {
            let mut futures: FuturesUnordered<_> = (0..10)
                .map(|i| async move {
                    sidevm::time::sleep(std::time::Duration::from_millis(i * 10)).await;
                })
                .collect();

            loop {
                info!("waiting for next future");
                let next = futures::StreamExt::next(&mut futures).await;
                if next.is_none() {
                    info!("All timers finished");
                    break;
                }
            }
        });
    }
    sidevm::time::sleep(std::time::Duration::from_secs(10)).await;
}
