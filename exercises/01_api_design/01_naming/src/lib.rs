//! # Exercise
//!
//! Rename this API so that a Rust programmer can guess it without reading it. Seven changes:
//!
//! | Now | Why it is wrong |
//! | --- | --- |
//! | `make_store` | constructors are called `new` |
//! | `set_value` | `HashMap` calls this `insert`, and `set` does not suggest a return value |
//! | `get_value` | `HashMap` calls this `get`, and `_value` says what a store holds anyway |
//! | `delete` | the standard library word is `remove` |
//! | `is_has_bucket` | `is_` is for adjectives, possession is `contains_` |
//! | `get_count` | `len`, and where there is a `len` there is an `is_empty` (add it) |
//! | `as_map` | `as_` promises a cheap borrow, this one clones the whole store |
//!
//! Only `is_empty` is new. Everything else keeps its body and changes its name.
//!
//! This exercise starts out **not compiling**: the tests are written against the names you are
//! supposed to arrive at.
//!
//! Afterwards the old names must be gone, not merely deprecated:
//!
//! ```compile_fail,E0599
//! use api_design_naming::Store;
//!
//! let store = Store::make_store();
//! ```

use std::collections::HashMap;

/// An in-memory key-value store, partitioned into named buckets.
pub struct Store {
    buckets: HashMap<String, HashMap<String, String>>,
}

impl Store {
    /// Creates an empty store.
    pub fn make_store() -> Store {
        Store {
            buckets: HashMap::new(),
        }
    }

    /// Stores a value, handing back whatever was there before.
    pub fn set_value(&mut self, bucket: &str, key: &str, value: &str) -> Option<String> {
        self.buckets
            .entry(bucket.to_owned())
            .or_default()
            .insert(key.to_owned(), value.to_owned())
    }

    /// Looks up a value.
    pub fn get_value(&self, bucket: &str, key: &str) -> Option<&str> {
        self.buckets.get(bucket)?.get(key).map(String::as_str)
    }

    /// Drops a value from its bucket, handing it back.
    pub fn delete(&mut self, bucket: &str, key: &str) -> Option<String> {
        self.buckets.get_mut(bucket)?.remove(key)
    }

    /// Reports whether a bucket exists.
    pub fn is_has_bucket(&self, bucket: &str) -> bool {
        self.buckets.contains_key(bucket)
    }

    /// Counts the values across every bucket.
    pub fn get_count(&self) -> usize {
        self.buckets.values().map(HashMap::len).sum()
    }

    /// Hands back the whole store as a plain map.
    pub fn as_map(&self) -> HashMap<String, HashMap<String, String>> {
        self.buckets.clone()
    }
}

#[cfg(test)]
mod tests {
    use crate::Store;

    #[test]
    fn insert_get_remove() {
        let mut store = Store::new();

        assert_eq!(store.insert("users", "42", "Alice"), None);
        assert_eq!(store.get("users", "42"), Some("Alice"));
        assert_eq!(store.insert("users", "42", "Bob").as_deref(), Some("Alice"));
        assert_eq!(store.remove("users", "42").as_deref(), Some("Bob"));
        assert_eq!(store.get("users", "42"), None);
    }

    #[test]
    fn len_and_is_empty_agree() {
        let mut store = Store::new();
        assert!(store.is_empty());
        assert_eq!(store.len(), 0);

        store.insert("users", "42", "Alice");
        store.insert("orders", "1", "a book");

        assert!(!store.is_empty());
        assert_eq!(store.len(), 2);
    }

    #[test]
    fn buckets_can_be_probed() {
        let mut store = Store::new();
        store.insert("users", "42", "Alice");

        assert!(store.contains_bucket("users"));
        assert!(!store.contains_bucket("orders"));
    }

    #[test]
    fn to_map_copies() {
        let mut store = Store::new();
        store.insert("users", "42", "Alice");

        let map = store.to_map();

        assert_eq!(map["users"]["42"], "Alice");
    }
}
