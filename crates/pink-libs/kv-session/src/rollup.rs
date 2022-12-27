use alloc::{
    collections::{BTreeMap, BTreeSet},
    vec::Vec,
};

use crate::traits::{BumpVersion, Key, KvSnapshot, KvTransaction, QueueIndex, Value};
use crate::Result;

pub enum VersionLayout {
    // Version stored in a seperate key with a magic postfix appended to the original key
    Standalone { key_postfix: Key },
    // Version stored inline together with value data
    // Inline,
}

#[derive(Debug)]
pub struct RollUpTransaction {
    /// Should be the block hash for a blockchain backend
    pub snapshot_id: Value,
    pub conditions: Vec<(Key, Option<Value>)>,
    pub updates: Vec<(Key, Option<Value>)>,
    pub queue_head: Option<QueueIndex>,
}

impl RollUpTransaction {
    /// Returns true if there are some updates in this transaction.
    pub fn has_updates(&self) -> bool {
        self.queue_head.is_some() || !self.updates.is_empty()
    }
}

/// Rollup version conditions and updates with given R/W tracking data and user updates
pub fn rollup<DB>(kvdb: DB, tx: KvTransaction, layout: VersionLayout) -> Result<RollUpTransaction>
where
    DB: KvSnapshot + BumpVersion,
{
    let VersionLayout::Standalone {
        key_postfix: postfix,
    } = layout;

    fn concat(a: &[u8], b: &[u8]) -> Vec<u8> {
        [a, b].concat()
    }

    let lookup_keys: BTreeSet<_> = tx
        .accessed_keys
        .iter()
        .chain(tx.version_updates.iter())
        .cloned()
        .collect();

    let version_keys: Vec<_> = lookup_keys
        .into_iter()
        .map(|k| concat(&k, &postfix))
        .collect();

    let versions: BTreeMap<_, _> = kvdb.batch_get(&version_keys)?.into_iter().collect();

    let mut conditions: Vec<_> = tx
        .accessed_keys
        .into_iter()
        .map(|k| {
            (
                concat(&k, &postfix),
                versions.get(&concat(&k, &postfix)).and_then(Clone::clone),
            )
        })
        .collect();

    if let Some(lock_key) = tx.queue_lock {
        let lock_value = kvdb.get(&lock_key)?;
        conditions.push((lock_key, lock_value));
    }

    let mut updates = tx.value_updates;
    {
        // Auto bump the versions of keys we have written to
        let version_updates: Result<Vec<_>> = tx
            .version_updates
            .iter()
            .map(|k| {
                let key = concat(k, &postfix);
                let ver = versions.get(&key).and_then(Clone::clone);
                kvdb.bump_version(ver).map(|ver| (key, Some(ver)))
            })
            .collect();
        let mut version_updates = version_updates?;
        updates.append(&mut version_updates);
    }

    Ok(RollUpTransaction {
        conditions,
        updates,
        snapshot_id: kvdb.snapshot_id()?,
        queue_head: tx.queue_head,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use alloc::{borrow::ToOwned, sync::Arc, vec};
    use core::cell::RefCell;
    use scale::{Decode, Encode};

    use crate::{
        traits::{KvSession, QueueIndexCodec, QueueSession, Value},
        Error, ReadTracker, Session,
    };

    #[derive(Clone, Default)]
    struct MockSnapshot {
        db: Arc<RefCell<BTreeMap<Vec<u8>, Vec<u8>>>>,
    }

    impl MockSnapshot {
        fn set(&self, key: &[u8], value: &[u8]) {
            self.db.borrow_mut().insert(key.to_vec(), value.to_vec());
        }
    }

    impl KvSnapshot for MockSnapshot {
        fn get(&self, key: &[u8]) -> Result<Option<Value>> {
            let key = key.to_owned();
            Ok(self.db.borrow().get(&key).cloned())
        }

        fn snapshot_id(&self) -> Result<Value> {
            Ok(Default::default())
        }
    }

    impl BumpVersion for MockSnapshot {
        fn bump_version(&self, version: Option<Value>) -> Result<Value> {
            match version {
                Some(v) => {
                    let ov = u32::decode(&mut &v[..]).or(Err(Error::FailedToDecode))?;
                    Ok((ov + 1).encode())
                }
                None => Ok(1_u32.encode()),
            }
        }
    }

    struct ScaleCodec;
    impl QueueIndexCodec for ScaleCodec {
        fn encode(number: u32) -> Vec<u8> {
            number.encode()
        }

        fn decode(raw: impl AsRef<[u8]>) -> Result<u32> {
            Decode::decode(&mut raw.as_ref()).or(Err(Error::FailedToDecode))
        }
    }

    #[test]
    fn rollup_works() {
        let kvdb = MockSnapshot::default();
        // Set up test data
        kvdb.set(b"A", &1_u32.encode());
        //  Set version of B
        kvdb.set(b"B_ver", &10000_u32.encode());

        let tx = {
            // An operation in a session
            let mut session =
                Session::<_, _, ScaleCodec>::new(kvdb.clone(), ReadTracker::new(), b"_q/").unwrap();
            // op seq:
            // get A
            // get B
            // set B
            // set C
            // del B

            assert_eq!(session.get(b"A").unwrap(), Some(1_u32.encode()));
            assert_eq!(session.get(b"B").unwrap(), None);

            session.put(b"B", 24_u32.encode());
            assert_eq!(session.get(b"B").unwrap(), Some(24_u32.encode()));

            session.put(b"C", 34_u32.encode());
            assert_eq!(session.get(b"C").unwrap(), Some(34_u32.encode()));

            session.delete(b"B");
            assert_eq!(session.get(b"B").unwrap(), None);

            session.commit().0
        };

        let rollup = rollup(
            kvdb,
            tx,
            VersionLayout::Standalone {
                key_postfix: "_ver".into(),
            },
        )
        .unwrap();
        assert_eq!(
            rollup.conditions,
            [
                (b"A_ver".to_vec(), None), // Require version of A to be None
                (b"B_ver".to_vec(), Some(10000_u32.encode()))  // Require version of B to be 10000
            ]
        );
        assert_eq!(
            rollup.updates,
            [
                (b"B".to_vec(), None),                         // Delete value of key B
                (b"C".to_vec(), Some(34_u32.encode())),        // Update value of key C to 34
                (b"B_ver".to_vec(), Some(10001_u32.encode())), // Update version of B from 10000 to 10001
                (b"C_ver".to_vec(), Some(1_u32.encode())), // Update version of C from `None` to `1`
            ]
        );
    }

    #[test]
    fn empty_queue_works() {
        let kvdb = MockSnapshot::default();
        let mut queue =
            Session::<_, _, ScaleCodec>::new(kvdb, ReadTracker::new(), b"TestQ/").unwrap();
        assert_eq!(queue.queue_length(), 0);
        assert_eq!(queue.pop(), Ok(None));
        let (tx, kvdb) = queue.commit();

        let tx = rollup(
            kvdb,
            tx,
            VersionLayout::Standalone {
                key_postfix: "_ver".into(),
            },
        )
        .unwrap();
        // Should not lock the "TestQ/head" due to empty queue
        assert_eq!(tx.conditions, vec![]);
        assert_eq!(tx.updates, vec![]);
        assert_eq!(tx.queue_head, None);
    }

    #[test]
    fn pop_queue_works() {
        let kvdb = MockSnapshot::default();

        // Set up some test data
        kvdb.set(b"TestQ/_head", &0_u128.encode());
        kvdb.set(b"TestQ/_tail", &2_u128.encode());
        kvdb.set(b"TestQ/\x00\x00\x00\x00", b"foo");
        kvdb.set(b"TestQ/\x01\x00\x00\x00", b"bar");

        let mut queue =
            Session::<_, _, ScaleCodec>::new(kvdb, ReadTracker::new(), b"TestQ/").unwrap();
        assert_eq!(queue.queue_length(), 2);
        assert_eq!(queue.pop(), Ok(Some(b"foo".to_vec())));
        assert_eq!(queue.pop(), Ok(Some(b"bar".to_vec())));
        assert_eq!(queue.pop(), Ok(None));
        let final_head = queue.queue_head();
        let (tx, kvdb) = queue.commit();
        let tx = rollup(
            kvdb,
            tx,
            VersionLayout::Standalone {
                key_postfix: "_ver".into(),
            },
        )
        .unwrap();

        assert_eq!(
            tx.conditions,
            // Should lock on the head cursor
            vec![(b"TestQ/_head".to_vec(), Some(0_u128.encode()))]
        );
        assert_eq!(tx.updates, vec![]);
        assert_eq!(tx.queue_head, Some(final_head));
    }
}
