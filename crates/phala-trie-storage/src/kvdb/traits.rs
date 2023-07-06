use parity_scale_codec::Encode;

use super::decode_value;

pub trait Transaction {
    fn get(&self, key: &[u8]) -> Option<Vec<u8>>;
    fn put(&self, key: &[u8], value: &[u8]);
    fn delete(&self, key: &[u8]);
    fn commit(self);
}

pub trait KvStorage {
    type Transaction<'a>: Transaction
    where
        Self: 'a;

    fn new() -> Self
    where
        Self: Sized;
    fn snapshot(&self) -> Self
    where
        Self: Sized;
    fn get(&self, key: &[u8]) -> Option<Vec<u8>>;
    fn transaction<'a>(&'a self) -> Self::Transaction<'a>;
    fn for_each(&self, cb: impl FnMut(&[u8], &[u8]));
    fn consolidate<K: AsRef<[u8]>>(&self, other: impl Iterator<Item = (K, (Vec<u8>, i32))>) {
        let transaction = self.transaction();
        for (key, (value, rc)) in other {
            if rc == 0 {
                continue;
            }

            let key = key.as_ref();

            let pv = transaction.get(key);
            let pv = decode_value(pv).expect("Failed to decode value");

            let raw_value = match pv {
                None => (value, rc),
                Some((mut d, mut orc)) => {
                    if orc <= 0 {
                        d = value;
                    }

                    orc += rc;

                    if orc == 0 {
                        transaction.delete(key);
                        continue;
                    }
                    (d, orc)
                }
            };
            transaction.put(key, &raw_value.encode());
        }
        transaction.commit();
    }
}
