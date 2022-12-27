use alloc::{
    collections::{BTreeMap, BTreeSet},
    vec::Vec,
};

use crate::traits::{BumpVersion, Concat, KvSnapshot, KvTransaction};
use crate::Result;

pub enum VersionLayout<Key> {
    // Version stored in a seperate key with a magic postfix appended to the original key
    Standalone { key_postfix: Key },
    // Version stored inline together with value data
    // Inline,
}

#[derive(Debug)]
pub struct RollUpTransaction<K, V> {
    /// Should be the block hash for a blockchain backend
    pub snapshot_id: V,
    pub conditions: Vec<(K, Option<V>)>,
    pub updates: Vec<(K, Option<V>)>,
}

impl<K: Concat + Clone, V> RollUpTransaction<K, V> {
    /// Give all keys in this transaction a prefix
    pub fn prefixed_with(self, prefix: K) -> Self {
        Self {
            conditions: self
                .conditions
                .into_iter()
                .map(|(k, v)| (prefix.clone().concat(k), v))
                .collect(),
            updates: self
                .updates
                .into_iter()
                .map(|(k, v)| (prefix.clone().concat(k), v))
                .collect(),
            snapshot_id: self.snapshot_id,
        }
    }
}

/// Rollup version conditions and updates with given R/W tracking data and user updates
pub fn rollup<K, V, DB>(
    kvdb: DB,
    tx: KvTransaction<K, V>,
    layout: VersionLayout<K>,
) -> Result<RollUpTransaction<K, V>>
where
    DB: KvSnapshot<Key = K, Value = V> + BumpVersion<V>,
    K: Concat + Clone + Ord,
    V: Clone,
{
    let VersionLayout::Standalone {
        key_postfix: postfix,
    } = layout;

    let lookup_keys: BTreeSet<_> = tx
        .accessed_keys
        .iter()
        .chain(tx.version_updates.iter())
        .cloned()
        .collect();

    let version_keys: Vec<_> = lookup_keys
        .into_iter()
        .map(|k| k.concat(postfix.clone()))
        .collect();

    let versions: BTreeMap<_, _> = kvdb.batch_get(&version_keys)?.into_iter().collect();

    let conditions: Vec<_> = tx
        .accessed_keys
        .into_iter()
        .map(|k| {
            (
                k.clone().concat(postfix.clone()),
                versions
                    .get(&k.concat(postfix.clone()))
                    .and_then(Clone::clone),
            )
        })
        .collect();

    let mut updates = tx.value_updates;
    {
        // Auto bump the versions of keys we have written to
        let version_updates: Result<Vec<_>> = tx
            .version_updates
            .iter()
            .map(|k| {
                let key = k.clone().concat(postfix.clone());
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
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use alloc::{borrow::ToOwned, string::String, sync::Arc};
    use core::cell::RefCell;

    use crate::{traits::KvSession, ReadTracker, Session};

    #[derive(Clone, Default)]
    struct MockSnapshot {
        db: Arc<RefCell<BTreeMap<String, u32>>>,
    }

    impl MockSnapshot {
        fn set(&self, key: String, value: u32) {
            self.db.borrow_mut().insert(key, value);
        }
    }

    impl KvSnapshot for MockSnapshot {
        type Key = String;

        type Value = u32;

        fn get(&self, key: &impl ToOwned<Owned = Self::Key>) -> Result<Option<Self::Value>> {
            let key = key.to_owned();
            Ok(self.db.borrow().get(&key).copied())
        }

        fn snapshot_id(&self) -> Result<Self::Value> {
            Ok(2022)
        }
    }

    impl BumpVersion<u32> for MockSnapshot {
        fn bump_version(&self, version: Option<u32>) -> Result<u32> {
            match version {
                Some(v) => Ok(v + 1),
                None => Ok(1),
            }
        }
    }

    #[test]
    fn rollup_works() {
        let kvdb = MockSnapshot::default();
        // Set up test data
        kvdb.set("A".into(), 1);
        //  Set version of B
        kvdb.set("B_ver".into(), 10000);

        let tx = {
            // An operation in a session
            let mut session = Session::new(kvdb.clone(), ReadTracker::new());
            // op seq:
            // get A
            // get B
            // set B
            // set C
            // del B

            assert_eq!(session.get("A").unwrap(), Some(1));
            assert_eq!(session.get("B").unwrap(), None);

            session.put("B", 24);
            assert_eq!(session.get("B").unwrap(), Some(24));

            session.put("C", 34);
            assert_eq!(session.get("C").unwrap(), Some(34));

            session.delete("B");
            assert_eq!(session.get("B").unwrap(), None);

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
                ("A_ver".to_owned(), None),        // Require version of A to be None
                ("B_ver".to_owned(), Some(10000))  // Require version of B to be 10000
            ]
        );
        assert_eq!(
            rollup.updates,
            [
                ("B".to_owned(), None),            // Delete value of key B
                ("C".to_owned(), Some(34)),        // Update value of key C to 34
                ("B_ver".to_owned(), Some(10001)), // Update version of B from 10000 to 10001
                ("C_ver".to_owned(), Some(1)),     // Update version of C from `None` to `1`
            ]
        );

        let rollup = rollup.prefixed_with("Foo_".into());
        assert_eq!(
            rollup.conditions,
            [
                ("Foo_A_ver".to_owned(), None),        // Require version of A to be None
                ("Foo_B_ver".to_owned(), Some(10000))  // Require version of B to be 10000
            ]
        );
        assert_eq!(
            rollup.updates,
            [
                ("Foo_B".to_owned(), None),            // Delete value of key B
                ("Foo_C".to_owned(), Some(34)),        // Update value of key C to 34
                ("Foo_B_ver".to_owned(), Some(10001)), // Update version of B from 10000 to 10001
                ("Foo_C_ver".to_owned(), Some(1)),     // Update version of C from `None` to `1`
            ]
        );
    }
}
