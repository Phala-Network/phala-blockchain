use alloc::vec;
use alloc::{collections::BTreeSet, vec::Vec};

use crate::traits::AccessTracking;

pub type ReadTracker<K> = AccessTracker<K, false>;
pub type RwTracker<K> = AccessTracker<K, true>;

/// Tracker that always emits the given one access history
pub struct OneLock<Key> {
    lock_key: Key,
    write_lock: bool,
}

impl<Key> OneLock<Key> {
    pub fn new(lock_key: Key, write_lock: bool) -> Self {
        Self {
            lock_key,
            write_lock,
        }
    }
}

impl<Key: Clone> AccessTracking for OneLock<Key> {
    type Key = Key;

    // One key tracker tracks nothing
    fn read(&mut self, _key: &Self::Key) {}

    fn write(&mut self, _key: &Self::Key) {}

    fn collect_into(self) -> (Vec<Self::Key>, Vec<Self::Key>) {
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
pub struct AccessTracker<Key, const TRACK_WRITE: bool> {
    track: BTreeSet<Key>,
    writes: BTreeSet<Key>,
}

impl<Key, const TRACK_WRITE: bool> AccessTracker<Key, TRACK_WRITE> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            track: Default::default(),
            writes: Default::default(),
        }
    }
}

impl<Key, const TRACK_WRITE: bool> AccessTracking for AccessTracker<Key, TRACK_WRITE>
where
    Key: Ord + Clone,
{
    type Key = Key;

    fn read(&mut self, key: &Self::Key) {
        self.track.insert(key.clone());
    }

    fn write(&mut self, key: &Self::Key) {
        self.writes.insert(key.clone());
        if TRACK_WRITE {
            self.track.insert(key.clone());
        }
    }

    fn collect_into(self) -> (Vec<Self::Key>, Vec<Self::Key>) {
        (
            self.track.into_iter().collect(),
            self.writes.into_iter().collect(),
        )
    }
}
