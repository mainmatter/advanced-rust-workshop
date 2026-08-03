//! # Exercise
//!
//! Nothing to write here. Read the code, run `wr`, and move on.
//!
//! This is `minidb` before anyone thought about names. Every method works. Every method compiles.
//! Every method is called something a reasonable person would not guess.
//!
//! Read the test at the bottom and count how many of those names you would have had to look up.

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
    fn everything_works_and_nothing_reads_well() {
        let mut store = Store::make_store();

        assert_eq!(store.set_value("users", "42", "Alice"), None);
        assert_eq!(store.get_value("users", "42"), Some("Alice"));
        assert!(store.is_has_bucket("users"));
        assert_eq!(store.get_count(), 1);
        assert_eq!(store.as_map()["users"]["42"], "Alice");
        assert_eq!(store.delete("users", "42").as_deref(), Some("Alice"));
    }
}
