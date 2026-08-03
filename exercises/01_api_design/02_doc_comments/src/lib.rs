//! # Exercise
//!
//! You renamed the API in the previous exercise. Nobody told the documentation.
//!
//! Three things are broken here, and each one fails differently:
//!
//! 1. `#![deny(missing_docs)]` is on, and three public methods have no documentation. The crate does
//!    not build until they do.
//! 2. Two examples still call the old names. Examples are compiled and run by `cargo test`, so they
//!    are the only part of a doc comment that cannot quietly rot. These have rotted anyway, which is
//!    what happens when nobody runs them.
//! 3. One doc comment describes behaviour this code does not have, and its example asserts the same
//!    falsehood. Fix the prose and the example together: decide which one is telling the truth.
//!
//! Write the missing docs the way the existing ones are written: a one-line summary that says what
//! the method is for, not how it works. Add an `# Examples` section only where an example earns its
//! keep.

#![deny(missing_docs)]

use std::collections::HashMap;

/// An in-memory key-value store, partitioned into named buckets.
pub struct Store {
    buckets: HashMap<String, HashMap<String, String>>,
}

impl Store {
    /// Creates an empty store.
    ///
    /// # Examples
    ///
    /// ```
    /// use api_design_doc_comments::Store;
    ///
    /// let store = Store::make_store();
    /// assert!(store.is_empty());
    /// ```
    pub fn new() -> Store {
        Store {
            buckets: HashMap::new(),
        }
    }

    /// Inserts a value, returning the value it replaced, if any.
    ///
    /// # Examples
    ///
    /// ```
    /// use api_design_doc_comments::Store;
    ///
    /// let mut store = Store::new();
    ///
    /// assert_eq!(store.set_value("users", "42", "Alice"), None);
    /// assert_eq!(store.set_value("users", "42", "Bob").as_deref(), Some("Alice"));
    /// ```
    pub fn insert(&mut self, bucket: &str, key: &str, value: &str) -> Option<String> {
        self.buckets
            .entry(bucket.to_owned())
            .or_default()
            .insert(key.to_owned(), value.to_owned())
    }

    /// Looks up a value, returning an empty string when the bucket or the key is unknown.
    ///
    /// # Examples
    ///
    /// ```
    /// use api_design_doc_comments::Store;
    ///
    /// let store = Store::new();
    ///
    /// assert_eq!(store.get("users", "42"), Some(""));
    /// ```
    pub fn get(&self, bucket: &str, key: &str) -> Option<&str> {
        self.buckets.get(bucket)?.get(key).map(String::as_str)
    }

    /// Removes a value, returning it if it was there.
    pub fn remove(&mut self, bucket: &str, key: &str) -> Option<String> {
        self.buckets.get_mut(bucket)?.remove(key)
    }

    pub fn contains_bucket(&self, bucket: &str) -> bool {
        self.buckets.contains_key(bucket)
    }

    pub fn len(&self) -> usize {
        self.buckets.values().map(HashMap::len).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Copies the whole store into a plain map.
    pub fn to_map(&self) -> HashMap<String, HashMap<String, String>> {
        self.buckets.clone()
    }
}

#[cfg(test)]
mod tests {
    use crate::Store;

    #[test]
    fn the_store_still_works() {
        let mut store = Store::new();
        store.insert("users", "42", "Alice");

        assert_eq!(store.get("users", "42"), Some("Alice"));
        assert_eq!(store.get("users", "43"), None);
        assert_eq!(store.get("orders", "42"), None);
        assert_eq!(store.len(), 1);
        assert!(store.contains_bucket("users"));
    }
}
