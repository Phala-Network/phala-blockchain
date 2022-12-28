use core::marker::PhantomData;

use alloc::vec::Vec;
use alloc::{borrow::ToOwned, collections::BTreeMap};

use crate::traits::{QueueIndex, QueueSession, QueueIndexCodec};
use crate::{
    traits::{AccessTracking, KvSession, KvSnapshot, KvTransaction},
    Result,
};

pub struct QueueInfo<Codec> {
    prefix: Vec<u8>,
    // The pos of first pushed message
    head: QueueIndex,
    // The pos to push the next message
    tail: QueueIndex,
    popped: bool,
    codec: PhantomData<Codec>,
}

impl<Codec: QueueIndexCodec> QueueInfo<Codec> {
    pub fn new<Snap: KvSnapshot>(prefix: impl Into<Vec<u8>>, snapshot: &Snap) -> Result<Self> {
        let prefix = prefix.into();
        macro_rules! get_number {
            ($key: expr) => {{
                let fullkey = [&prefix[..], &$key[..]].concat();
                snapshot.get(&fullkey)?.map(Codec::decode).transpose()
            }};
        }
        let head = get_number!(b"_head")?.unwrap_or(0);
        let tail = get_number!(b"_tail")?.unwrap_or(0);
        if tail < head {
            return Err(crate::Error::FailedToDecode);
        }

        Ok(Self {
            prefix,
            codec: PhantomData,
            head,
            tail,
            popped: false,
        })
    }

    pub fn pop(&mut self, kvdb: &impl KvSnapshot) -> Result<Option<Vec<u8>>> {
        if self.head == self.tail {
            return Ok(None);
        }
        let front_key = [&self.prefix[..], &Codec::encode(self.head)].concat();
        let data = kvdb.get(&front_key)?;
        self.popped = true;
        self.head += 1;
        Ok(data)
    }
}

pub struct Session<Snapshot, AccessTracker, Codec> {
    kvdb: Snapshot,
    tracker: AccessTracker,
    updates: BTreeMap<Vec<u8>, Option<Vec<u8>>>,
    queue: QueueInfo<Codec>,
}

impl<Snap, Tracker, Codec> Session<Snap, Tracker, Codec>
where
    Snap: KvSnapshot,
    Tracker: AccessTracking,
    Codec: QueueIndexCodec,
{
    pub fn new(kvdb: Snap, tracker: Tracker, queue_prefix: &[u8]) -> Result<Self> {
        let queue = QueueInfo::new(queue_prefix, &kvdb)?;
        Ok(Self {
            kvdb,
            tracker,
            updates: Default::default(),
            queue,
        })
    }

    pub fn commit(self) -> (KvTransaction, Snap) {
        let (accessed_keys, version_updates) = self.tracker.collect_into();
        (
            KvTransaction {
                accessed_keys,
                version_updates,
                value_updates: self.updates.into_iter().collect(),
                queue_head: if self.queue.popped {
                    Some(self.queue.head)
                } else {
                    None
                },
                queue_lock: if self.queue.popped {
                    Some([&self.queue.prefix[..], b"_head"].concat())
                } else {
                    None
                },
            },
            self.kvdb,
        )
    }

    #[cfg(test)]
    pub(crate) fn queue_head(&self) -> QueueIndex {
        self.queue.head
    }

    #[cfg(test)]
    pub(crate) fn queue_length(&self) -> QueueIndex {
        self.queue.tail - self.queue.head
    }
}

impl<Snap, Track, Codec> KvSession for Session<Snap, Track, Codec>
where
    Snap: KvSnapshot,
    Track: AccessTracking,
    Codec: QueueIndexCodec,
{
    fn get(&mut self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        if let Some(val) = self.updates.get(key) {
            return Ok(val.clone());
        }
        self.tracker.read(key);
        self.kvdb.get(key)
    }

    fn put(&mut self, key: &[u8], value: Vec<u8>) {
        let key = key.to_owned();
        self.tracker.write(&key);
        self.updates.insert(key, Some(value));
    }

    fn delete(&mut self, key: &[u8]) {
        let key = key.to_owned();
        self.tracker.write(&key);
        self.updates.insert(key, None);
    }
}

impl<Snap, Track, Codec> QueueSession for Session<Snap, Track, Codec>
where
    Snap: KvSnapshot,
    Track: AccessTracking,
    Codec: QueueIndexCodec,
{
    fn pop(&mut self) -> Result<Option<crate::traits::Value>> {
        self.queue.pop(&self.kvdb)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Error, OneLock, ReadTracker, RwTracker};
    use alloc::{sync::Arc, vec};
    use core::cell::RefCell;
    use scale::{Decode, Encode};

    #[derive(Clone, Default)]
    struct MockSnapshot {
        db: Arc<RefCell<BTreeMap<Vec<u8>, Vec<u8>>>>,
    }

    impl MockSnapshot {
        fn set(&self, key: &[u8], value: &[u8]) {
            self.db.borrow_mut().insert(key.into(), value.to_owned());
        }
    }

    impl KvSnapshot for MockSnapshot {
        fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
            let key = key.to_owned();
            Ok(self.db.borrow().get(&key).cloned())
        }

        fn snapshot_id(&self) -> Result<Vec<u8>> {
            Ok(vec![])
        }
    }

    struct ScaleCodec;
    impl QueueIndexCodec for ScaleCodec {
        fn encode(number: QueueIndex) -> Vec<u8> {
            Encode::encode(&number)
        }

        fn decode(raw: impl AsRef<[u8]>) -> Result<QueueIndex> {
            let mut buf = raw.as_ref();
            Decode::decode(&mut buf).or(Err(Error::FailedToDecode))
        }
    }

    fn with_tracker(tracker: impl AccessTracking) -> KvTransaction {
        let kvdb = MockSnapshot::default();
        kvdb.set(b"0", b"1");

        let mut session = Session::<_, _, ScaleCodec>::new(kvdb, tracker, b"_queue/").unwrap();

        assert_eq!(session.get(b"0").unwrap(), Some(b"1".to_vec()));
        assert_eq!(session.get(b"42").unwrap(), None);

        session.put(b"42", b"24".to_vec());
        assert_eq!(session.get(b"42").unwrap(), Some(b"24".to_vec()));

        session.put(b"43", b"34".to_vec());
        assert_eq!(session.get(b"43").unwrap(), Some(b"34".to_vec()));

        session.delete(b"42");
        assert_eq!(session.get(b"42").unwrap(), None);

        session.commit().0
    }

    #[test]
    fn session_with_one_lock_works() {
        let tx = with_tracker(OneLock::new(b"999", true));
        assert_eq!(tx.accessed_keys, vec![b"999"]);
        assert_eq!(
            tx.value_updates,
            vec![
                (b"42".to_vec(), None),
                (b"43".to_vec(), Some(b"34".to_vec())),
            ]
        );
        assert_eq!(tx.version_updates, vec![b"999"]);
    }

    #[test]
    fn session_with_read_tracker_works() {
        let tx = with_tracker(ReadTracker::new());
        assert_eq!(tx.accessed_keys, vec![b"0".to_vec(), b"42".to_vec()]);
        assert_eq!(
            tx.value_updates,
            vec![
                (b"42".to_vec(), None),
                (b"43".to_vec(), Some(b"34".to_vec())),
            ]
        );
        assert_eq!(tx.version_updates, vec![b"42".to_vec(), b"43".to_vec()]);
    }

    #[test]
    fn session_with_rw_tracker_works() {
        let tx = with_tracker(RwTracker::new());
        assert_eq!(
            tx.accessed_keys,
            vec![b"0".to_vec(), b"42".to_vec(), b"43".to_vec()]
        );
        assert_eq!(
            tx.value_updates,
            vec![
                (b"42".to_vec(), None),
                (b"43".to_vec(), Some(b"34".to_vec())),
            ]
        );
        assert_eq!(tx.version_updates, vec![b"42".to_vec(), b"43".to_vec()]);
    }
}
