#[cfg_attr(not(feature = "std"), no_std)]
use core::alloc::{GlobalAlloc, Layout};
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

pub struct StatSizeAllocator<T> {
    inner: T,
    current: AtomicUsize,
    spike: AtomicUsize,
    peak: AtomicUsize,
    #[cfg(feature = "track")]
    track_enabled: AtomicBool,
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
            #[cfg(feature = "track")]
            track_enabled: AtomicBool::new(false),
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

    #[cfg(feature = "track")]
    pub fn enable_track(&self) {
        self.track_enabled.store(true, Ordering::Relaxed);
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

    #[cfg(feature = "track")]
    fn track_alloc(&self, ptr: *mut u8, size: usize) -> *mut u8 {
        if !self.track_enabled.load(Ordering::Relaxed) {
            return ptr;
        }
        with_tracking(|| {
            println!("alloc: {ptr:?} {size}");
            println!("stack trace: {:#?}", std::backtrace::Backtrace::force_capture());
            let stack = format!("{:?}", std::backtrace::Backtrace::force_capture());
            if size == 116 && stack.contains("tracing_core::dispatcher::get_default") {
                panic!("bad alloc");
            }
        });
        ptr
    }

    #[cfg(feature = "track")]
    fn track_dealloc(&self, ptr: *mut u8, size: usize) {
        if !self.track_enabled.load(Ordering::Relaxed) {
            return;
        }
        with_tracking(|| {
            println!("dealloc: {ptr:?} {size}");
            println!("stack trace: {:#?}", std::backtrace::Backtrace::force_capture());
        });
    }

    #[cfg(not(feature = "track"))]
    fn track_alloc(&self, ptr: *mut u8) -> *mut u8 {
        ptr
    }

    #[cfg(not(feature = "track"))]
    fn track_dealloc(&self, _ptr: *mut u8) {}
}

fn with_tracking<F, R>(f: F) -> Option<R>
where
    F: FnOnce() -> R,
{
    std::thread_local! {
        pub static TRACKING: std::cell::Cell<bool> = Default::default();
    }
    TRACKING.with(|t| {
        if t.get() {
            None
        } else {
            t.set(true);
            let r = f();
            t.set(false);
            Some(r)
        }
    })
}

unsafe impl<T: GlobalAlloc> GlobalAlloc for StatSizeAllocator<T> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.add_alloced_size(layout.size());
        self.track_alloc(self.inner.alloc(layout), layout.size())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.current.fetch_sub(layout.size(), Ordering::SeqCst);
        self.inner.dealloc(ptr, layout);
        self.track_dealloc(ptr, layout.size());
    }

    #[cfg(not(feature = "track"))]
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        self.add_alloced_size(layout.size());
        self.inner.alloc_zeroed(layout)
    }

    #[cfg(not(feature = "track"))]
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
