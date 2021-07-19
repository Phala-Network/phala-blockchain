use crate::std;

use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use log::debug;

// TODO.kevin: block_box will do best-effort to prevent compiler optimizations, but not guaranteed.
use core::hint::black_box;

const UNIT: usize = 1;
const MAX_NUM: u128 = 65536 * 128;

static ITERATION_COUNTER: AtomicU64 = AtomicU64::new(0);
static PAUSED: AtomicBool = AtomicBool::new(false);
static SCORE: AtomicU64 = AtomicU64::new(0);

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
    let since = now();
    let start = iteration_counter();
    loop {
        for _ in 0..UNIT {
            let _ = black_box(count_prime(black_box(MAX_NUM)));
        }
        let count = ITERATION_COUNTER.fetch_add(1, Ordering::Relaxed);
        if count % 100 == 0 {
            let score = est_score(since, start);
            debug!(
                "Benchmark counnter increased to {}, est score={}",
                count, score,
            );
            SCORE.store(score, Ordering::Relaxed);
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

pub fn score() -> u64 {
    SCORE.load(Ordering::Relaxed)
}

fn est_score(since: u64, start: u64) -> u64 {
    let now = now();
    if now <= since {
        return 0;
    }
    // Normalize to 6s (standard block time)
    (ITERATION_COUNTER.load(Ordering::Relaxed) - start) * 6 / (now - since)
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Get time failed")
        .as_secs()
}
