use futures::future::try_join_all;
use log::{debug, error};
use tokio::task::JoinHandle;

pub static CONTENT_TYPE_JSON: &str = "application/json";
pub static CONTENT_TYPE_BIN: &str = "application/octet-stream";

pub async fn join_handles(handles: Vec<JoinHandle<()>>) {
    match try_join_all(handles).await {
        Ok(_) => {
            debug!("Joint task finished.");
        }
        Err(err) => {
            error!("Fatal error: {}", err);
            std::process::exit(100);
        }
    }
}

#[macro_export]
macro_rules! with_retry {
    ($f:expr, $c:expr, $s:expr) => {{
        let mut retry_count: u64 = 0;
        loop {
            let r = $f.await;
            match r {
                Err(e) => {
                    warn!("Attempt #{retry_count}({}): {}", stringify!($f), &e);
                    retry_count += 1;
                    if (retry_count - 1) == $c {
                        break Err(e);
                    }
                    tokio::time::sleep(std::time::Duration::from_millis($s)).await;
                }
                _ => break r,
            }
        }
    }};
}
