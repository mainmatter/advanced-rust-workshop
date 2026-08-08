//! # Exercise
//!
//! `Bucket` and `Key` are distinct types now, but they still accept anything. A key can be empty,
//! or a megabyte of user-supplied bytes, and nothing stops it.
//!
//! Implement `Bucket::parse` and `Key::parse`. A name is valid when it is:
//!
//! - not empty;
//! - at most 64 bytes long;
//! - made up exclusively of ASCII alphanumerics and `-`, `_`, `.`, `/`.
//!
//! Check those rules in that order, and report the first violation. Write the validation once and
//! call it from both types.
//!
//! `is_valid_char` is already in the file: it is that third rule, spelled out. Use it rather than
//! writing the character set again.
//!
//! Then **delete `Bucket::new` and `Key::new`**. Parsing is worthless while the unchecked door is
//! still open, so this must stop compiling:
//!
//! ```compile_fail,E0599
//! use newtype_parse_dont_validate::Key;
//!
//! let key = Key::new("42");
//! ```

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

    /// Parses a bucket name, rejecting anything `minidb` cannot store.
    pub fn parse(raw: &str) -> Result<Self, NameError> {
        todo!()
    }
}

/// The name of a value within a bucket.
pub struct Key(pub String);

impl Key {
    /// Creates a key.
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    /// Parses a key, rejecting anything `minidb` cannot store.
    pub fn parse(raw: &str) -> Result<Self, NameError> {
        todo!()
    }
}

/// What can go wrong when parsing a bucket name or a key.
#[derive(Debug, PartialEq, Eq)]
pub enum NameError {
    Empty,
    TooLong { length: usize },
    InvalidCharacter { character: char, index: usize },
}

fn is_valid_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '/')
}

#[cfg(test)]
mod tests {
    use crate::{Bucket, Key, MAX_NAME_LENGTH, NameError, Store};

    #[test]
    fn valid_names_are_accepted() {
        assert!(Bucket::parse("users").is_ok());
        assert!(Key::parse("users/42").is_ok());
        assert!(Key::parse("a-b_c.d/e9").is_ok());
        assert!(Key::parse(&"x".repeat(MAX_NAME_LENGTH)).is_ok());
    }

    #[test]
    fn empty_names_are_rejected() {
        assert_eq!(Bucket::parse("").err(), Some(NameError::Empty));
        assert_eq!(Key::parse("").err(), Some(NameError::Empty));
    }

    #[test]
    fn long_names_are_rejected() {
        let raw = "x".repeat(MAX_NAME_LENGTH + 1);

        assert_eq!(
            Key::parse(&raw).err(),
            Some(NameError::TooLong {
                length: MAX_NAME_LENGTH + 1
            })
        );
    }

    #[test]
    fn invalid_characters_are_rejected() {
        assert_eq!(
            Key::parse("user 42").err(),
            Some(NameError::InvalidCharacter {
                character: ' ',
                index: 4
            })
        );
        assert_eq!(
            Key::parse("caf\u{e9}").err(),
            Some(NameError::InvalidCharacter {
                character: '\u{e9}',
                index: 3
            })
        );
    }

    #[test]
    fn the_first_violation_wins() {
        let raw = format!("{} ", "x".repeat(MAX_NAME_LENGTH));

        assert_eq!(
            Key::parse(&raw).err(),
            Some(NameError::TooLong {
                length: MAX_NAME_LENGTH + 1
            })
        );
    }

    #[test]
    fn parsed_names_still_work_as_names() {
        let mut store = Store::new();
        let users = Bucket::parse("users").unwrap();
        let id = Key::parse("42").unwrap();

        store.insert(&users, &id, "Alice");

        assert_eq!(store.get(&users, &id), Some("Alice"));
    }
}
