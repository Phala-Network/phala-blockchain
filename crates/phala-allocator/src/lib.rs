#![no_std]
use core::alloc::{GlobalAlloc, Layout};
use core::sync::atomic::{AtomicUsize, Ordering};

pub struct StatSizeAllocator<T> {
    inner: T,
    current_used: AtomicUsize,
    peak_used: AtomicUsize,
}

#[derive(Debug)]
pub struct Stats {
    /// The current heap usage of the allocator.
    pub current_used: usize,
    /// The peak heap usage of the allocator.
    pub peak_used: usize,
}

impl<T> StatSizeAllocator<T> {
    pub const fn new(inner: T) -> Self {
        Self {
            inner,
            current_used: AtomicUsize::new(0),
            peak_used: AtomicUsize::new(0),
        }
    }

    pub fn stats(&self) -> Stats {
        Stats {
            current_used: self.current_used.load(Ordering::Relaxed),
            peak_used: self.peak_used.load(Ordering::Relaxed),
        }
    }
}

impl<T: GlobalAlloc> StatSizeAllocator<T> {
    fn add_alloced_size(&self, size: usize) {
        let prev = self.current_used.fetch_add(size, Ordering::SeqCst);
        let total_size = prev + size;
        let mut peak = self.peak_used.load(Ordering::SeqCst);
        loop {
            if total_size <= peak {
                break;
            }
            match self.peak_used.compare_exchange(
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
        self.current_used.fetch_sub(layout.size(), Ordering::SeqCst);
        self.inner.dealloc(ptr, layout)
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        self.add_alloced_size(layout.size());
        self.inner.alloc_zeroed(layout)
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        if new_size > layout.size() {
            let difference = new_size - layout.size();
            self.add_alloced_size(difference);
        } else if new_size < layout.size() {
            let difference = layout.size() - new_size;
            self.current_used.fetch_sub(difference, Ordering::SeqCst);
        }
        self.inner.realloc(ptr, layout, new_size)
    }
}
