pub fn ordmap_for_each_mut<K: Ord + Clone, V: Clone>(
    map: &mut im::OrdMap<K, V>,
    mut f: impl FnMut((&K, &mut V)),
) {
    let snapshot = map.clone();
    for key in snapshot.keys() {
        f((key, map.get_mut(key).unwrap()));
    }
}
