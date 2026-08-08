//! # Exercise
//!
//! Nothing to write here. Read the doctests, run `wr`, and move on.
//!
//! `Key` is exactly the type we wanted at the end of the last chapter: it cannot be confused with a
//! `Bucket`, it cannot hold an invalid name, and it cannot be built without going through `parse`.
//!
//! It also cannot be printed, compared, or used as a key in a `HashMap`, which is an odd thing to
//! say about a type called `Key`. All three of these are proof, not prose:
//!
//! ```compile_fail,E0277
//! use common_traits_intro::Key;
//!
//! let key = Key::parse("users/42").unwrap();
//! println!("{key:?}");
//! ```
//!
//! ```compile_fail,E0369
//! use common_traits_intro::Key;
//!
//! let a = Key::parse("users/42").unwrap();
//! let b = Key::parse("users/42").unwrap();
//! let same = a == b;
//! ```
//!
//! ```compile_fail,E0277
//! use common_traits_intro::Key;
//! use std::collections::HashMap;
//!
//! let mut map = HashMap::new();
//! map.insert(Key::parse("users/42").unwrap(), "Alice");
//! ```
//!
//! A `String` does all three. Wrapping it took them away, and this chapter is the bill.

use std::collections::HashMap;

const MAX_NAME_LENGTH: usize = 64;

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
            .entry(bucket.as_str().to_owned())
            .or_default()
            .insert(key.as_str().to_owned(), value.to_owned())
    }

    /// Looks up a value.
    pub fn get(&self, bucket: &Bucket, key: &Key) -> Option<&str> {
        self.buckets
            .get(bucket.as_str())?
            .get(key.as_str())
            .map(String::as_str)
    }

    /// Removes a value, returning it if it was there.
    pub fn remove(&mut self, bucket: &Bucket, key: &Key) -> Option<String> {
        self.buckets.get_mut(bucket.as_str())?.remove(key.as_str())
    }
}

/// The name of a bucket.
pub struct Bucket(String);

impl Bucket {
    /// Parses a bucket name, rejecting anything `minidb` cannot store.
    pub fn parse(raw: &str) -> Result<Self, NameError> {
        parse_name(raw).map(Self)
    }

    /// Borrows the bucket name.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes the bucket name, returning the wrapped `String`.
    pub fn into_inner(self) -> String {
        self.0
    }
}

/// The name of a value within a bucket.
pub struct Key(String);

impl Key {
    /// Parses a key, rejecting anything `minidb` cannot store.
    pub fn parse(raw: &str) -> Result<Self, NameError> {
        parse_name(raw).map(Self)
    }

    /// Borrows the key.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes the key, returning the wrapped `String`.
    pub fn into_inner(self) -> String {
        self.0
    }
}

/// What can go wrong when parsing a bucket name or a key.
#[derive(Debug, PartialEq, Eq)]
pub enum NameError {
    Empty,
    TooLong { length: usize },
    InvalidCharacter { character: char, index: usize },
}

fn parse_name(raw: &str) -> Result<String, NameError> {
    if raw.is_empty() {
        return Err(NameError::Empty);
    }

    if raw.len() > MAX_NAME_LENGTH {
        return Err(NameError::TooLong { length: raw.len() });
    }

    match raw.char_indices().find(|(_, c)| !is_valid_char(*c)) {
        Some((index, character)) => Err(NameError::InvalidCharacter { character, index }),
        None => Ok(raw.to_owned()),
    }
}

fn is_valid_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '/')
}

#[cfg(test)]
mod tests {
    use crate::{Bucket, Key, Store};

    #[test]
    fn the_store_still_works() {
        let mut store = Store::new();
        let users = Bucket::parse("users").unwrap();
        let id = Key::parse("42").unwrap();

        store.insert(&users, &id, "Alice");

        assert_eq!(store.get(&users, &id), Some("Alice"));
    }
}
