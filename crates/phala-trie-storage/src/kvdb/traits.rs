use parity_scale_codec::Encode;

use super::decode_value;

pub(crate) trait Transaction {
    type Error: std::fmt::Debug;
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error>;
    fn put(&self, key: &[u8], value: &[u8]) -> Result<(), Self::Error>;
    fn delete(&self, key: &[u8]) -> Result<(), Self::Error>;
    fn commit(self) -> Result<(), Self::Error>;
}

pub(crate) trait KvStorage {
    type Transaction<'a>: Transaction
    where
        Self: 'a;

    fn transaction<'a>(&'a self) -> Self::Transaction<'a>;

    fn consolidate<K: AsRef<[u8]>>(&self, other: impl Iterator<Item = (K, (Vec<u8>, i32))>) {
        let transaction = self.transaction();
        for (key, (value, rc)) in other {
            if rc == 0 {
                continue;
            }

            let key = key.as_ref();

            let pv = transaction.get(key).expect("Failed to get value from DB");
            let pv = decode_value(pv).expect("Failed to decode value");

            let raw_value = match pv {
                None => (value, rc),
                Some((mut d, mut orc)) => {
                    if orc <= 0 {
                        d = value;
                    }

                    orc += rc;

                    if orc == 0 {
                        transaction
                            .delete(key)
                            .expect("Failed to delete key from transaction");
                        continue;
                    }
                    (d, orc)
                }
            };
            transaction
                .put(key, &raw_value.encode())
                .expect("Failed to put key in transaction");
        }
        transaction.commit().expect("Failed to commit transaction");
    }
}
