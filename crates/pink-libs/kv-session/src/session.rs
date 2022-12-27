use alloc::{borrow::ToOwned, collections::BTreeMap};

use crate::{
    traits::{AccessTracking, KvSession, KvSnapshot, KvTransaction},
    Result,
};

pub struct Session<Snapshot, Key, Value, AccessTracker> {
    kvdb: Snapshot,
    tracker: AccessTracker,
    updates: BTreeMap<Key, Option<Value>>,
}

impl<Snap, Key, Val, Tracker> Session<Snap, Key, Val, Tracker>
where
    Snap: KvSnapshot<Key = Key, Value = Val>,
    Key: Ord + Clone,
    Tracker: AccessTracking<Key = Key>,
{
    pub fn new(kvdb: Snap, tracker: Tracker) -> Self {
        Self {
            kvdb,
            tracker,
            updates: Default::default(),
        }
    }

    pub fn commit(self) -> (KvTransaction<Key, Val>, Snap) {
        let (accessed_keys, version_updates) = self.tracker.collect_into();
        (
            KvTransaction {
                accessed_keys,
                version_updates,
                value_updates: self.updates.into_iter().collect(),
            },
            self.kvdb,
        )
    }
}

impl<Snap, Key, Val, Track> KvSession for Session<Snap, Key, Val, Track>
where
    Snap: KvSnapshot<Key = Key, Value = Val>,
    Key: Ord + Clone,
    Val: Clone,
    Track: AccessTracking<Key = Key>,
{
    type Key = Key;
    type Value = Val;

    fn get(&mut self, key: &(impl ToOwned<Owned = Key> + ?Sized)) -> Result<Option<Val>> {
        let key = key.to_owned();
        if let Some(val) = self.updates.get(&key) {
            return Ok(val.clone());
        }
        self.tracker.read(&key);
        self.kvdb.get(&key)
    }

    fn put(&mut self, key: &(impl ToOwned<Owned = Key> + ?Sized), value: Val) {
        let key = key.to_owned();
        self.tracker.write(&key);
        self.updates.insert(key, Some(value));
    }

    fn delete(&mut self, key: &(impl ToOwned<Owned = Key> + ?Sized)) {
        let key = key.to_owned();
        self.tracker.write(&key);
        self.updates.insert(key, None);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{OneLock, ReadTracker, RwTracker};
    use alloc::vec;

    struct MockSnapshot;

    impl KvSnapshot for MockSnapshot {
        type Key = u32;

        type Value = u32;

        fn get(&self, key: &impl ToOwned<Owned = Self::Key>) -> Result<Option<Self::Value>> {
            let key = key.to_owned();
            if key < 42 {
                return Ok(Some(key + 1));
            }
            Ok(None)
        }

        fn snapshot_id(&self) -> Result<Self::Value> {
            Ok(0)
        }
    }

    fn with_tracker(tracker: impl AccessTracking<Key = u32>) -> KvTransaction<u32, u32> {
        let mut session = Session::new(MockSnapshot, tracker);

        assert_eq!(session.get(&0).unwrap(), Some(1));
        assert_eq!(session.get(&42).unwrap(), None);

        session.put(&42, 24);
        assert_eq!(session.get(&42).unwrap(), Some(24));

        session.put(&43, 34);
        assert_eq!(session.get(&43).unwrap(), Some(34));

        session.delete(&42);
        assert_eq!(session.get(&42).unwrap(), None);

        session.commit().0
    }

    #[test]
    fn session_with_one_lock_works() {
        let tx = with_tracker(OneLock::new(999, true));
        assert_eq!(tx.accessed_keys, vec![999]);
        assert_eq!(tx.value_updates, vec![(42, None), (43, Some(34)),]);
        assert_eq!(tx.version_updates, vec![999]);
    }

    #[test]
    fn session_with_read_tracker_works() {
        let tx = with_tracker(ReadTracker::new());
        assert_eq!(tx.accessed_keys, vec![0, 42]);
        assert_eq!(tx.value_updates, vec![(42, None), (43, Some(34)),]);
        assert_eq!(tx.version_updates, vec![42, 43]);
    }

    #[test]
    fn session_with_rw_tracker_works() {
        let tx = with_tracker(RwTracker::new());
        assert_eq!(tx.accessed_keys, vec![0, 42, 43]);
        assert_eq!(tx.value_updates, vec![(42, None), (43, Some(34)),]);
        assert_eq!(tx.version_updates, vec![42, 43]);
    }
}
