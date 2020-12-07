use std::{
    borrow::Borrow,
    collections::HashSet,
    hash::Hash,
    ops::Deref,
    sync::{Arc, RwLock, Weak},
};

pub struct RodHashMap<K, V>
where
    K: Eq + Hash,
{
    inner: Arc<RwLock<HashSet<RodEntry<K, V>>>>,
}

impl<K: Eq + Hash, V> RodHashMap<K, V> {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashSet::new())),
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
    K: Eq + Hash,
{
    key: Arc<K>,
    value: Weak<RodGuard<K, V>>,
}

impl<K: Eq + Hash, V> RodEntry<K, V> {
    fn new(parent: Arc<RwLock<HashSet<Self>>>, key: K, value: V) -> (Self, Arc<RodGuard<K, V>>) {
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
            .expect("If value is dropped this should NOT still be accessible")
    }
}

impl<K: Eq + Hash, V> Borrow<K> for RodEntry<K, V> {
    fn borrow(&self) -> &K {
        &self.key
    }
}

impl<K: Eq + Hash, V> PartialEq for RodEntry<K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.key.eq(&other.key)
    }
}

impl<K: Eq + Hash, V> Eq for RodEntry<K, V> {}

impl<K: Eq + Hash, V> Hash for RodEntry<K, V> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.key.hash(state);
    }
}

pub struct RodGuard<K, V>
where
    K: Eq + Hash,
{
    parent: Arc<RwLock<HashSet<RodEntry<K, V>>>>,
    key: Arc<K>,
    value: V,
}

impl<K: Eq + Hash, V> RodGuard<K, V> {
    fn new(parent: Arc<RwLock<HashSet<RodEntry<K, V>>>>, key: Arc<K>, value: V) -> Self {
        Self { parent, key, value }
    }
}

impl<K: Eq + Hash, V> Deref for RodGuard<K, V> {
    type Target = V;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<K: Eq + Hash, V> Drop for RodGuard<K, V> {
    fn drop(&mut self) {
        self.parent.write().unwrap().remove(&*self.key);
    }
}

#[cfg(test)]
mod tests {
    use super::RodHashMap;

    #[test]
    fn single_guard() {
        struct Room;

        let mut hotel = RodHashMap::<&str, Room>::new();

        assert!(hotel.is_empty());

        let room_0 = hotel.insert("Room Number 0", Room);

        assert_eq!(hotel.len(), 1);

        drop(room_0);

        assert!(hotel.is_empty());
    }

    #[test]
    fn cloned_guard() {
        struct Room;

        let mut hotel = RodHashMap::<&str, Room>::new();

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

        let mut hotel = RodHashMap::<&str, Room>::new();

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
