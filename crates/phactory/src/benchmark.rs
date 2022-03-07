use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use log::debug;
use serde::{Deserialize, Serialize};

// TODO.kevin: block_box will do best-effort to prevent compiler optimizations, but not guaranteed.
use core::hint::black_box;

const UNIT: usize = 1;
const MAX_NUM: u128 = 65536 * 128;

static ITERATION_COUNTER: AtomicU64 = AtomicU64::new(0);
static PAUSED: AtomicBool = AtomicBool::new(true);
static SCORE: AtomicU64 = AtomicU64::new(0);

// If pruntime is synchronizing the history blocks, we consider it as not ready to benchmark.
static READY_TO_BENCH: AtomicBool = AtomicBool::new(true);

#[derive(Serialize, Deserialize, Debug)]
pub struct State {
    pub counter: u64,
    pub score: u64,
    pub paused: bool,
}

pub fn dump_state() -> State {
    State {
        counter: ITERATION_COUNTER.load(Ordering::Relaxed),
        score: SCORE.load(Ordering::Relaxed),
        paused: PAUSED.load(Ordering::Relaxed),
    }
}

pub fn restore_state(state: State) {
    ITERATION_COUNTER.store(state.counter, Ordering::Relaxed);
    SCORE.store(state.score, Ordering::Relaxed);
    PAUSED.store(state.paused, Ordering::Relaxed);
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
        if puasing() {
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
    PAUSED.load(Ordering::Relaxed) || !READY_TO_BENCH.load(Ordering::Relaxed)
}

pub fn set_ready(ready: bool) {
    READY_TO_BENCH.store(ready, Ordering::Relaxed)
}

pub fn is_ready() -> bool {
    READY_TO_BENCH.load(Ordering::Relaxed)
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
