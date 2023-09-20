#[cfg(not(test))]
pub use im::OrdMap;
#[cfg(test)]
pub use std::collections::BTreeMap as OrdMap;

pub fn ordmap_for_each_mut<K: Ord + Clone, V: Clone>(
    map: &mut OrdMap<K, V>,
    mut f: impl FnMut((&K, &mut V)),
) {
    let snapshot = map.clone();
    for key in snapshot.keys() {
        f((key, map.get_mut(key).unwrap()));
    }
}
