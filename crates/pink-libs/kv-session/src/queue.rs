use alloc::vec::Vec;
use core::marker::PhantomData;

use crate::{
    rollup::RollUpTransaction,
    traits::{KvSession, KvSnapshot, KvSnapshotExt, PrefixedKvSnapshot},
    OneLock, Result, Session,
};

type QueueIndex = u32;

/// A tx committed from a queue session.
pub struct QueueTransaction {
    /// The inner tx from the underlying kv session.
    pub inner_tx: RollUpTransaction<Vec<u8>, Vec<u8>>,
    /// The final queue head.
    pub queue_head: QueueIndex,
}

pub trait QueueIndexCodec {
    type Index;
    fn encode(number: Self::Index) -> Vec<u8>;
    fn decode(raw: impl AsRef<[u8]>) -> Result<Self::Index>;
}

type InnerSession<Snap> =
    Session<PrefixedKvSnapshot<Vec<u8>, Snap>, Vec<u8>, Vec<u8>, OneLock<Vec<u8>>>;

pub struct MessageQueueSession<Snap, Cod> {
    pub session: InnerSession<Snap>,
    prefix: Vec<u8>,
    // The pos of first pushed message
    head: QueueIndex,
    // The pos to push the next message
    tail: QueueIndex,
    codec: PhantomData<Cod>,
}

impl<S, C> MessageQueueSession<S, C>
where
    S: KvSnapshot<Key = Vec<u8>, Value = Vec<u8>> + Clone,
    C: QueueIndexCodec<Index = QueueIndex>,
{
    pub fn new(prefix: impl Into<Vec<u8>>, snapshot: S) -> Result<Self> {
        let prefix = prefix.into();
        let session = Session::new(
            snapshot.prefixed(prefix.clone()),
            // Treat the head cursor as lock
            OneLock::new(b"_head".to_vec(), false),
        );

        Self {
            prefix,
            session,
            codec: PhantomData,
            head: 0,
            tail: 0,
        }
        .init()
    }

    fn init(mut self) -> Result<Self> {
        self.head = self.get_number(b"_head")?.unwrap_or(0);
        self.tail = self.get_number(b"_tail")?.unwrap_or(0);
        if self.tail < self.head {
            return Err(crate::Error::FailedToDecode);
        }
        Ok(self)
    }

    fn get_number(&mut self, key: &[u8; 5]) -> Result<Option<QueueIndex>> {
        self.session.get(&key[..])?.map(C::decode).transpose()
    }

    pub fn length(&self) -> QueueIndex {
        self.tail - self.head
    }

    pub fn pop(&mut self) -> Result<Option<Vec<u8>>> {
        if self.head == self.tail {
            return Ok(None);
        }
        let front_key = C::encode(self.head);
        let data = self.session.get(&front_key)?;
        self.head += 1;
        Ok(data)
    }

    pub fn commit(self) -> Result<QueueTransaction> {
        let (tx, snapshot) = self.session.commit();

        let conditions = snapshot.batch_get(&tx.accessed_keys)?;
        let snapshot_id = snapshot.snapshot_id()?;
        Ok(QueueTransaction {
            inner_tx: RollUpTransaction {
                snapshot_id,
                conditions,
                updates: tx.value_updates,
            }
            .prefixed_with(self.prefix),
            queue_head: self.head,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use alloc::{borrow::ToOwned, collections::BTreeMap, sync::Arc, vec};
    use core::cell::RefCell;
    use scale::{Decode, Encode};

    use crate::Error;

    #[derive(Clone, Default)]
    struct MockSnapshot {
        db: Arc<RefCell<BTreeMap<Vec<u8>, Vec<u8>>>>,
    }

    impl MockSnapshot {
        fn set(&self, key: impl Into<Vec<u8>>, value: &[u8]) {
            self.db.borrow_mut().insert(key.into(), value.to_owned());
        }
    }

    impl KvSnapshot for MockSnapshot {
        type Key = Vec<u8>;

        type Value = Vec<u8>;

        fn get(&self, key: &impl ToOwned<Owned = Self::Key>) -> Result<Option<Self::Value>> {
            let key = key.to_owned();
            Ok(self.db.borrow().get(&key).cloned())
        }

        fn snapshot_id(&self) -> Result<Self::Value> {
            Ok(vec![])
        }
    }

    struct ScaleCodec;
    impl QueueIndexCodec for ScaleCodec {
        type Index = u32;
        fn encode(number: Self::Index) -> Vec<u8> {
            Encode::encode(&number)
        }

        fn decode(raw: impl AsRef<[u8]>) -> Result<Self::Index> {
            let mut buf = raw.as_ref();
            Decode::decode(&mut buf).or(Err(Error::FailedToDecode))
        }
    }

    #[test]
    fn empty_queue_works() {
        let kvdb = MockSnapshot::default();
        let mut queue = MessageQueueSession::<_, ScaleCodec>::new("TestQ/", kvdb).unwrap();
        assert_eq!(queue.length(), 0);
        assert_eq!(queue.pop(), Ok(None));
        let tx = queue.commit().unwrap();
        // Should lock the "TestQ/head"
        assert_eq!(
            tx.inner_tx.conditions,
            vec![(b"TestQ/_head".to_vec(), None)]
        );
        assert_eq!(tx.inner_tx.updates, vec![]);
        assert_eq!(tx.queue_head, 0);
    }

    #[test]
    fn pop_queue_works() {
        let kvdb = MockSnapshot::default();

        // Set up some test data
        kvdb.set(*b"TestQ/_head", &0_u128.encode());
        kvdb.set(*b"TestQ/_tail", &2_u128.encode());
        kvdb.set(*b"TestQ/\x00\x00\x00\x00", b"foo");
        kvdb.set(*b"TestQ/\x01\x00\x00\x00", b"bar");

        let mut queue = MessageQueueSession::<_, ScaleCodec>::new(*b"TestQ/", kvdb).unwrap();
        assert_eq!(queue.length(), 2);
        assert_eq!(queue.pop(), Ok(Some(b"foo".to_vec())));
        assert_eq!(queue.pop(), Ok(Some(b"bar".to_vec())));
        assert_eq!(queue.pop(), Ok(None));
        let final_head = queue.head;
        let tx = queue.commit().unwrap();

        assert_eq!(
            tx.inner_tx.conditions,
            // Should lock on the head cursor
            vec![(b"TestQ/_head".to_vec(), Some(0_u128.encode()))]
        );
        assert_eq!(tx.inner_tx.updates, vec![]);
        assert_eq!(tx.queue_head, final_head);
    }
}
