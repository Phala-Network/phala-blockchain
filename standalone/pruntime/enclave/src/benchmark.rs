use crate::std;

use core::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use log::debug;

// TODO.kevin: block_box will do best-effort to prevent compiler optimizations, but not guaranteed.
use core::hint::black_box;

const UNIT: usize = 1;
const MAX_NUM: u128 = 65536 * 128;

static ITERATION_COUNTER: AtomicU64 = AtomicU64::new(0);
static COUNT_SINCE: AtomicU64 = AtomicU64::new(0);
static PAUSED: AtomicBool = AtomicBool::new(false);

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

pub fn run() {
    loop {
        for _ in 0..UNIT {
            let _ = black_box(count_prime(black_box(MAX_NUM)));
        }
        let count = ITERATION_COUNTER.fetch_add(1, Ordering::Relaxed);
        if count % 100 == 0 {
            debug!(
                "Benchmark counnter increased to {}, est score={}",
                count,
                est_score()
            );
        }
        if PAUSED.load(Ordering::Relaxed) {
            return;
        }
    }
}

pub fn iteration_counter() -> u64 {
    ITERATION_COUNTER.load(Ordering::Relaxed)
}

pub fn reset_iteration_counter() {
    ITERATION_COUNTER.store(0, Ordering::Relaxed);
    if debugging() {
        COUNT_SINCE.store(now(), Ordering::Relaxed);
    }
}

pub fn pause() {
    PAUSED.store(true, Ordering::Relaxed)
}

pub fn resume() {
    PAUSED.store(false, Ordering::Relaxed)
}

pub fn puasing() -> bool {
    PAUSED.load(Ordering::Relaxed)
}

fn est_score() -> u64 {
    let since = COUNT_SINCE.load(Ordering::Relaxed);
    let now = now();
    if now <= since {
        return 0;
    }
    ITERATION_COUNTER.load(Ordering::Relaxed) / (now - since)
}

fn debugging() -> bool {
    log::STATIC_MAX_LEVEL >= log::Level::Debug && log::max_level() >= log::Level::Debug
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Get time failed")
        .as_secs()
}
