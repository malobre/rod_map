use std::{
    borrow::Borrow,
    collections::BTreeSet,
    ops::Deref,
    sync::{Arc, RwLock, Weak},
};

pub struct RodBTreeMap<K, V>
where
    K: Ord + Eq,
{
    inner: Arc<RwLock<BTreeSet<RodEntry<K, V>>>>,
}

impl<K: Ord + Eq, V> RodBTreeMap<K, V> {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(BTreeSet::new())),
        }
    }

    pub fn len(&self) -> usize {
        self.inner.read().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.read().unwrap().is_empty()
    }

    pub fn insert(&mut self, key: K, value: V) -> Arc<RodGuard<K, V>> {
        let (entry, guard) = RodEntry::new(Arc::clone(&self.inner), key, value);
        self.inner.write().unwrap().insert(entry);

        guard
    }

    pub fn get(&self, key: &K) -> Option<Arc<RodGuard<K, V>>> {
        self.inner.read().unwrap().get(key).map(|entry| entry.get())
    }
}

struct RodEntry<K, V>
where
    K: Ord + Eq,
{
    key: Arc<K>,
    value: Weak<RodGuard<K, V>>,
}

impl<K: Ord + Eq, V> RodEntry<K, V> {
    fn new(parent: Arc<RwLock<BTreeSet<Self>>>, key: K, value: V) -> (Self, Arc<RodGuard<K, V>>) {
        let key = Arc::new(key);
        let guard = Arc::new(RodGuard::new(parent, Arc::clone(&key), value));

        (
            Self {
                key,
                value: Arc::downgrade(&guard),
            },
            guard,
        )
    }

    fn get(&self) -> Arc<RodGuard<K, V>> {
        self.value
            .upgrade()
            .expect("Value was dropped, this should NOT still be accessible")
    }
}

impl<K: Ord + Eq, V> PartialEq for RodEntry<K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.key.eq(&other.key)
    }
}

impl<K: Ord + Eq, V> Eq for RodEntry<K, V> {}

impl<K: Ord + Eq, V> PartialOrd for RodEntry<K, V> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.key.partial_cmp(&other.key)
    }
}

impl<K: Ord + Eq, V> Ord for RodEntry<K, V> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.key.cmp(&other.key)
    }
}

impl<K: Ord, V> Borrow<K> for RodEntry<K, V> {
    fn borrow(&self) -> &K {
        &self.key
    }
}

pub struct RodGuard<K, V>
where
    K: Ord,
{
    parent: Arc<RwLock<BTreeSet<RodEntry<K, V>>>>,
    key: Arc<K>,
    value: V,
}

impl<K, V> RodGuard<K, V>
where
    K: Ord,
{
    fn new(parent: Arc<RwLock<BTreeSet<RodEntry<K, V>>>>, key: Arc<K>, value: V) -> Self {
        Self { parent, key, value }
    }
}

impl<K, V> Deref for RodGuard<K, V>
where
    K: Ord,
{
    type Target = V;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<K, V> Drop for RodGuard<K, V>
where
    K: Ord,
{
    fn drop(&mut self) {
        self.parent.write().unwrap().remove(&*self.key);
    }
}

#[cfg(test)]
mod tests {
    use super::RodBTreeMap;

    #[test]
    fn single_guard() {
        struct Room;

        let mut hotel = RodBTreeMap::<&str, Room>::new();

        assert!(hotel.is_empty());

        let room_0 = hotel.insert("Room Number 0", Room);

        assert_eq!(hotel.len(), 1);

        drop(room_0);

        assert!(hotel.is_empty());
    }

    #[test]
    fn cloned_guard() {
        struct Room;

        let mut hotel = RodBTreeMap::<&str, Room>::new();

        assert!(hotel.is_empty());

        let room_0 = hotel.insert("Room Number 0", Room);
        let room_0_clone = room_0.clone();

        assert_eq!(hotel.len(), 1);

        drop(room_0);

        assert_eq!(hotel.len(), 1);

        drop(room_0_clone);

        assert!(hotel.is_empty());
    }

    #[test]
    fn insert_and_get() {
        struct Room;

        let mut hotel = RodBTreeMap::<&str, Room>::new();

        assert!(hotel.is_empty());

        let room_0_key = "Room Number 0";
        let room_0_from_insert = hotel.insert(room_0_key, Room);
        let room_0_from_get = hotel.get(&room_0_key).unwrap();

        assert_eq!(hotel.len(), 1);

        drop(room_0_from_insert);

        assert_eq!(hotel.len(), 1);

        drop(room_0_from_get);

        assert!(hotel.is_empty());
    }
}
