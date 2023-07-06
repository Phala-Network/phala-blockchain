use core::marker::PhantomData;
use hash_db::Hasher;
use parity_scale_codec::{Decode, Encode};
use serde::{
    de::{SeqAccess, Visitor},
    ser::SerializeSeq,
    Deserializer, Serializer,
};

use crate::kvdb::{traits::Transaction, DecodedDBValue};

use super::traits::KvStorage;

pub(crate) fn deserialize<'de, D, DB: KvStorage, H: Hasher>(deserializer: D) -> Result<DB, D::Error>
where
    D: Deserializer<'de>,
{
    struct VecVisitor<H, DB> {
        marker: PhantomData<(H, DB)>,
    }
    impl<'de, H: Hasher, DB: KvStorage> Visitor<'de> for VecVisitor<H, DB> {
        type Value = DB;

        fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
            formatter.write_str("a sequence")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let db = DB::new();
            let transaction = db.transaction();
            while let Some((value, rc)) = seq.next_element::<DecodedDBValue>()? {
                let key = H::hash(&value);
                transaction.put(key.as_ref(), &(value, rc).encode());
            }
            transaction.commit();
            Ok(db)
        }
    }
    deserializer.deserialize_seq(VecVisitor {
        marker: PhantomData::<(H, DB)>,
    })
}

pub(crate) fn serialize<S, DB>(db: &DB, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    DB: KvStorage,
{
    let mut seq = serializer.serialize_seq(None)?;
    db.for_each(|_k, v| {
        let element: DecodedDBValue =
            Decode::decode(&mut &v[..]).expect("Failed to decode db value");
        seq.serialize_element(&element)
            .expect("Failed to serialize element");
    });
    seq.end()
}
