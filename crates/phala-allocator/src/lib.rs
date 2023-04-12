#![no_std]
use core::alloc::{GlobalAlloc, Layout};
use core::sync::atomic::{AtomicUsize, Ordering};

pub struct StatSizeAllocator<T> {
    inner: T,
    current: AtomicUsize,
    spike: AtomicUsize,
    peak: AtomicUsize,
}

#[derive(Debug)]
pub struct Stats {
    /// The current heap usage of the allocator.
    pub current: usize,
    /// The peak heap usage of the allocator in a short-term duration.
    pub spike: usize,
    /// The peak heap usage of the allocator.
    pub peak: usize,
}

impl<T> StatSizeAllocator<T> {
    pub const fn new(inner: T) -> Self {
        Self {
            inner,
            current: AtomicUsize::new(0),
            spike: AtomicUsize::new(0),
            peak: AtomicUsize::new(0),
        }
    }

    pub fn stats(&self) -> Stats {
        let spike = self.spike.swap(0, Ordering::Relaxed);
        let current_peak = self.peak.load(Ordering::Relaxed);
        let peak = current_peak.max(spike);
        self.peak.store(peak, Ordering::Relaxed);
        Stats {
            current: self.current.load(Ordering::Relaxed),
            spike,
            peak,
        }
    }
}

impl<T: GlobalAlloc> StatSizeAllocator<T> {
    fn add_alloced_size(&self, size: usize) {
        let prev = self.current.fetch_add(size, Ordering::SeqCst);
        let total_size = prev + size;
        let mut peak = self.spike.load(Ordering::SeqCst);
        loop {
            if total_size <= peak {
                break;
            }
            match self.spike.compare_exchange(
                peak,
                total_size,
                Ordering::Acquire,
                Ordering::Relaxed,
            ) {
                Err(new) => {
                    peak = new;
                    continue;
                }
                Ok(_) => {
                    break;
                }
            }
        }
    }
}

unsafe impl<T: GlobalAlloc> GlobalAlloc for StatSizeAllocator<T> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.add_alloced_size(layout.size());
        self.inner.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.current.fetch_sub(layout.size(), Ordering::SeqCst);
        self.inner.dealloc(ptr, layout)
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        self.add_alloced_size(layout.size());
        self.inner.alloc_zeroed(layout)
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        use core::cmp::Ordering::*;
        match new_size.cmp(&layout.size()) {
            Less => {
                let difference = layout.size() - new_size;
                self.current.fetch_sub(difference, Ordering::SeqCst);
            }
            Greater => {
                let difference = new_size - layout.size();
                self.add_alloced_size(difference);
            }
            Equal => (),
        }
        self.inner.realloc(ptr, layout, new_size)
    }
}
