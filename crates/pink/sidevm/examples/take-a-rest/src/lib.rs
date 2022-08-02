use log::info;

use pink_sidevm as sidevm;
use sidevm::{logger::Logger, ocall};
use std::ptr;

pub fn black_box<T>(dummy: T) -> T {
    unsafe {
        let ret = ptr::read_volatile(&dummy);
        std::mem::forget(dummy);
        ret
    }
}

fn is_prime(num: u128) -> bool {
    let tmp = num - 1;
    for i in tmp..=2 {
        if num % black_box(i) == 0 {
            return false;
        }
    }
    true
}

fn count_prime(max: u128) -> usize {
    let mut count = 0;
    for i in 2..max {
        if is_prime(black_box(i)) {
            count += 1;
        }
    }
    count
}

pub async fn run() {
    const MAX_NUM: u128 = 65536 * 16;
    const UNIT: usize = 1;

    let since = now();
    let start = 0;
    let mut counter = 0;
    loop {
        for _ in 0..UNIT {
            let _ = black_box(count_prime(black_box(MAX_NUM)));
        }
        counter += 1;
        let score = est_score(since, start, counter) as f32 / 1000.0;
        log::info!("Bench counter={counter}, est score={score:.3}");
        sidevm::time::maybe_rest().await;
    }
}

fn est_score(since: u64, start: u64, counter: u64) -> u64 {
    let now = now();
    if now <= since {
        return 0;
    }
    // Normalize to 6s (standard block time)
    (counter - start) * (6000 / 8) / (now - since)
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Get time failed")
        .as_secs()
}

#[sidevm::main]
async fn main() {
    Logger::with_max_level(log::LevelFilter::Trace).init();
    ocall::enable_ocall_trace(true).unwrap();
    info!("starting...");
    run().await;
}
