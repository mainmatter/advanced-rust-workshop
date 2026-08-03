//! # Exercise
//!
//! Nothing to write here. Read the code, run `wr`, and move on.
//!
//! The store you renamed in the previous chapter, trimmed to the four operations we will spend the
//! rest of the day evolving. It works, the tests pass, and it is the kind of code that ships every
//! day.
//!
//! Pay attention to the second test. It passes.

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
    pub fn insert(&mut self, bucket: &str, key: &str, value: &str) -> Option<String> {
        self.buckets
            .entry(bucket.to_owned())
            .or_default()
            .insert(key.to_owned(), value.to_owned())
    }

    /// Looks up a value.
    pub fn get(&self, bucket: &str, key: &str) -> Option<&str> {
        self.buckets.get(bucket)?.get(key).map(String::as_str)
    }

    /// Removes a value, returning it if it was there.
    pub fn remove(&mut self, bucket: &str, key: &str) -> Option<String> {
        self.buckets.get_mut(bucket)?.remove(key)
    }
}

#[cfg(test)]
mod tests {
    use crate::Store;

    #[test]
    fn insert_and_get() {
        let mut store = Store::new();
        store.insert("users", "42", "Alice");

        assert_eq!(store.get("users", "42"), Some("Alice"));
        assert_eq!(store.remove("users", "42").as_deref(), Some("Alice"));
        assert_eq!(store.get("users", "42"), None);
    }

    #[test]
    fn swapping_the_arguments_compiles_and_lies() {
        let mut store = Store::new();
        store.insert("users", "42", "Alice");

        assert_eq!(store.get("42", "users"), None);
    }
}
