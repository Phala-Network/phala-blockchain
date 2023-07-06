use core::marker::PhantomData;
use hash_db::Hasher;
use parity_scale_codec::{Decode, Encode};
use serde::{
    de::{MapAccess, SeqAccess, Visitor},
    ser::{SerializeMap, SerializeSeq},
    Deserializer, Serializer,
};

use crate::kvdb::{traits::Transaction, DecodedDBValue};

use super::traits::KvStorage;

pub(crate) fn deserialize_from_seq<'de, D, DB: KvStorage, H: Hasher>(
    deserializer: D,
) -> Result<DB, D::Error>
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

pub(crate) fn serialize_as_seq<S, DB>(db: &DB, serializer: S) -> Result<S::Ok, S::Error>
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

pub(crate) fn deserialize_from_map<'de, D, DB: KvStorage>(
    deserializer: D,
) -> Result<DB, D::Error>
where
    D: Deserializer<'de>,
{
    struct MapVisitor<DB>(PhantomData<DB>);
    impl<'de, DB: KvStorage> Visitor<'de> for MapVisitor<DB> {
        type Value = DB;

        fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
            formatter.write_str("a map")
        }

        fn visit_map<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            let db = DB::new();
            let transaction = db.transaction();
            while let Some((key, (rc, value))) = seq.next_entry::<Vec<u8>, (i32, Vec<u8>)>()? {
                transaction.put(key.as_ref(), &(value, rc).encode());
            }
            transaction.commit();
            Ok(db)
        }
    }
    deserializer.deserialize_map(MapVisitor(PhantomData::<DB>))
}

pub(crate) fn serialize_as_map<S, DB>(db: &DB, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    DB: KvStorage,
{
    let mut ser = serializer.serialize_map(None)?;
    db.for_each(|key, v| {
        let (value, rc): DecodedDBValue =
            Decode::decode(&mut &v[..]).expect("Failed to decode db value");
        ser.serialize_entry(key, &(rc, value))
            .expect("Failed to serialize element");
    });
    ser.end()
}
