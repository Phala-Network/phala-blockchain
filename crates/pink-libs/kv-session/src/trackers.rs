use alloc::vec;
use alloc::{collections::BTreeSet, vec::Vec};

use crate::traits::{AccessTracking, Key};

pub type ReadTracker = AccessTracker<false>;
pub type RwTracker = AccessTracker<true>;

/// Tracker that always emits the given one access history
pub struct OneLock {
    lock_key: Key,
    write_lock: bool,
}

impl OneLock {
    pub fn new(lock_key: &[u8], write_lock: bool) -> Self {
        Self {
            lock_key: lock_key.to_vec(),
            write_lock,
        }
    }
}

impl AccessTracking for OneLock {
    // One key tracker tracks nothing
    fn read(&mut self, _key: &[u8]) {}

    fn write(&mut self, _key: &[u8]) {}

    fn collect_into(self) -> (Vec<Key>, Vec<Key>) {
        (
            vec![self.lock_key.clone()],
            if self.write_lock {
                vec![self.lock_key]
            } else {
                vec![]
            },
        )
    }
}

/// Tracker that emit read and(optional) write access history
pub struct AccessTracker<const TRACK_WRITE: bool> {
    track: BTreeSet<Key>,
    writes: BTreeSet<Key>,
}

impl<const TRACK_WRITE: bool> AccessTracker<TRACK_WRITE> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            track: Default::default(),
            writes: Default::default(),
        }
    }
}

impl<const TRACK_WRITE: bool> AccessTracking for AccessTracker<TRACK_WRITE> {
    fn read(&mut self, key: &[u8]) {
        self.track.insert(key.to_vec());
    }

    fn write(&mut self, key: &[u8]) {
        self.writes.insert(key.to_vec());
        if TRACK_WRITE {
            self.track.insert(key.to_vec());
        }
    }

    fn collect_into(self) -> (Vec<Key>, Vec<Key>) {
        (
            self.track.into_iter().collect(),
            self.writes.into_iter().collect(),
        )
    }
}
