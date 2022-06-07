use futures::{
    channel::mpsc::channel,
    stream::futures_unordered::FuturesUnordered, SinkExt, StreamExt,
};
use log::info;

use pink_sidevm as sidevm;
use sidevm::{logger::Logger, ocall};

#[sidevm::main]
async fn main() {
    Logger::with_max_level(log::Level::Trace).init();
    ocall::enable_ocall_trace(true).unwrap();
    info!("creating channel...");
    let (tx, mut rx) = channel(100);
    info!("starting...");
    for tid in 0..20 {
        let mut tx = tx.clone();
        sidevm::spawn(async move {
            let mut futures: FuturesUnordered<_> = (0..3)
                .map(|i| async move {
                    sidevm::time::sleep(std::time::Duration::from_millis(i * 10)).await;
                })
                .collect();

            loop {
                info!("waiting for next future");
                let next = futures::StreamExt::next(&mut futures).await;
                if next.is_none() {
                    break;
                }
            }
            tx.send(tid).await.unwrap();
            info!("All timers finished");
        });
    }
    drop(tx);
    loop {
        let tid = rx.next().await;
        match tid {
            None => {
                info!("All tasks finished");
                break;
            }
            Some(tid) => info!("Task {} finished", tid+1),
        }
    }
}
