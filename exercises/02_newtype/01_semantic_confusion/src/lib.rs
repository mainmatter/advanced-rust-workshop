//! # Exercise
//!
//! Two `&str` parameters in a row, meaning different things. Make the compiler tell them apart.
//!
//! Introduce two newtypes and thread them through `Store`:
//!
//! ```text
//! Bucket::new(impl Into<String>) -> Bucket
//! Key::new(impl Into<String>) -> Key
//!
//! Store::insert(&mut self, bucket: &Bucket, key: &Key, value: &str) -> Option<String>
//! Store::get(&self, bucket: &Bucket, key: &Key) -> Option<&str>
//! Store::remove(&mut self, bucket: &Bucket, key: &Key) -> Option<String>
//! ```
//!
//! Keep the wrapped value public for now (`pub struct Key(pub String)`). We will come back to that.
//!
//! This exercise starts out **not compiling**: the tests below are written against the API you are
//! supposed to build, and they cannot find it yet. Read them, then build it.
//!
//! Once you are done, this must not compile any more, which is the whole point:
//!
//! ```compile_fail,E0308
//! use newtype_semantic_confusion::{Bucket, Key, Store};
//!
//! let mut store = Store::new();
//! let users = Bucket::new("users");
//! let id = Key::new("42");
//!
//! store.insert(&users, &id, "Alice");
//! store.get(&id, &users);
//! ```
//!
//! A type alias will not do. `type Key = String;` compiles, and changes nothing.

use std::collections::HashMap;

/// An in-memory key-value store, partitioned into named buckets.
pub struct Store {
    buckets: HashMap<String, HashMap<String, String>>,
}

impl Store {
    /// Creates an empty store.
    pub fn new() -> Self {
        Self {
            buckets: HashMap::new(),
        }
    }

    /// Inserts a value, returning the value it replaced, if any.
    pub fn insert(&mut self, bucket: &Bucket, key: &Key, value: &str) -> Option<String> {
        self.buckets
            .entry(bucket.0.clone())
            .or_default()
            .insert(key.0.clone(), value.to_owned())
    }

    /// Looks up a value.
    pub fn get(&self, bucket: &Bucket, key: &Key) -> Option<&str> {
        self.buckets.get(&bucket.0)?.get(&key.0).map(String::as_str)
    }

    /// Removes a value, returning it if it was there.
    pub fn remove(&mut self, bucket: &Bucket, key: &Key) -> Option<String> {
        self.buckets.get_mut(&bucket.0)?.remove(&key.0)
    }
}

/// The name of a bucket.
pub struct Bucket(pub String);

impl Bucket {
    /// Creates a bucket name.
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }
}

/// The name of a value within a bucket.
pub struct Key(pub String);

impl Key {
    /// Creates a key.
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }
}

#[cfg(test)]
mod tests {
    use crate::{Bucket, Key, Store};

    #[test]
    fn insert_and_get() {
        let mut store = Store::new();
        let users = Bucket::new("users");
        let id = Key::new("42");

        assert_eq!(store.insert(&users, &id, "Alice"), None);
        assert_eq!(store.get(&users, &id), Some("Alice"));
        assert_eq!(store.insert(&users, &id, "Bob").as_deref(), Some("Alice"));
        assert_eq!(store.remove(&users, &id).as_deref(), Some("Bob"));
        assert_eq!(store.get(&users, &id), None);
    }

    #[test]
    fn buckets_are_independent() {
        let mut store = Store::new();
        let users = Bucket::new("users");
        let orders = Bucket::new("orders");
        let id = Key::new("42");

        store.insert(&users, &id, "Alice");

        assert_eq!(store.get(&orders, &id), None);
    }
}
