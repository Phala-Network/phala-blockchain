use log::debug;
use core::sync::atomic::{AtomicU64, Ordering};

// TODO.kevin: block_box will do best-effort to prevent compiler optimizations, but not guaranteed.
use core::hint::black_box;

const UNIT: usize = 1;
const MAX_NUM: u128 = 65536*128;

static ITERATION_COUNTER: AtomicU64 =  AtomicU64::new(0);

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

pub fn run() -> ! {
    loop {
        for _ in 0..UNIT {
            let _ = black_box(count_prime(black_box(MAX_NUM)));
        }
        let count = ITERATION_COUNTER.fetch_add(1, Ordering::Relaxed);
        if count % 100 == 0 {
            debug!("Benchmark counnter increased to {}", count);
        }
    }
}

pub fn iteration_counter() -> u64 {
    ITERATION_COUNTER.load(Ordering::Relaxed)
}

pub fn reset_iteration_counter() {
    ITERATION_COUNTER.store(0, Ordering::Relaxed)
}