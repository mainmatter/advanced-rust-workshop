//! # Exercise
//!
//! `Key::parse` rejects invalid keys. `Key("".to_owned())` builds one anyway, and so does
//! `key.0.push(' ')`. The invariant you just wrote holds for exactly as long as nobody reaches around
//! it, which in a shared codebase is about a week.
//!
//! Close the hole:
//!
//! - make the wrapped `String` private in both `Bucket` and `Key`;
//! - implement `as_str(&self) -> &str` for borrowed access;
//! - implement `into_inner(self) -> String` for callers who want the name back;
//! - and switch `Store` over to `as_str`, which currently reaches straight into the field.
//!
//! Note what `into_inner` does and does not give away: a `String` you can do anything with, but no way
//! to put it back into a `Key` without going through `parse`. Giving out `&mut String` would be a
//! different story, which is why neither type does.
//!
//! Resist `impl Deref<Target = String>`. It would make every `String` method show up on `Key`,
//! including the ones that break it.
//!
//! These two must stop compiling:
//!
//! ```compile_fail,E0603
//! use newtype_encapsulation::Key;
//!
//! let key = Key("not a valid key, and not parsed either".to_owned());
//! ```
//!
//! ```compile_fail,E0616
//! use newtype_encapsulation::Key;
//!
//! let key = Key::parse("users/42").unwrap();
//! let raw = &key.0;
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
    /// Parses a bucket name, rejecting anything `minidb` cannot store.
    pub fn parse(raw: &str) -> Result<Self, NameError> {
        parse_name(raw).map(Self)
    }

    /// Borrows the bucket name.
    pub fn as_str(&self) -> &str {
        todo!()
    }

    /// Consumes the bucket name, returning the wrapped `String`.
    pub fn into_inner(self) -> String {
        todo!()
    }
}

/// The name of a value within a bucket.
pub struct Key(pub String);

impl Key {
    /// Parses a key, rejecting anything `minidb` cannot store.
    pub fn parse(raw: &str) -> Result<Self, NameError> {
        parse_name(raw).map(Self)
    }

    /// Borrows the key.
    pub fn as_str(&self) -> &str {
        todo!()
    }

    /// Consumes the key, returning the wrapped `String`.
    pub fn into_inner(self) -> String {
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

fn parse_name(raw: &str) -> Result<String, NameError> {
    if raw.is_empty() {
        return Err(NameError::Empty);
    }

    if raw.len() > MAX_NAME_LENGTH {
        return Err(NameError::TooLong { length: raw.len() });
    }

    match raw.char_indices().find(|(_, c)| !is_allowed(*c)) {
        Some((index, character)) => Err(NameError::InvalidCharacter { character, index }),
        None => Ok(raw.to_owned()),
    }
}

fn is_allowed(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '/')
}

#[cfg(test)]
mod tests {
    use crate::{Bucket, Key, MAX_NAME_LENGTH, NameError, Store};

    #[test]
    fn as_str_borrows_the_name() {
        let bucket = Bucket::parse("users").unwrap();
        let key = Key::parse("users/42").unwrap();

        assert_eq!(bucket.as_str(), "users");
        assert_eq!(key.as_str(), "users/42");
    }

    #[test]
    fn into_inner_gives_the_name_back() {
        let bucket = Bucket::parse("users").unwrap();
        let key = Key::parse("users/42").unwrap();

        assert_eq!(bucket.into_inner(), "users".to_owned());
        assert_eq!(key.into_inner(), "users/42".to_owned());
    }

    #[test]
    fn valid_names_are_accepted() {
        assert!(Bucket::parse("users").is_ok());
        assert!(Key::parse("users/42").is_ok());
        assert!(Key::parse("a-b_c.d/e9").is_ok());
        assert!(Key::parse(&"x".repeat(MAX_NAME_LENGTH)).is_ok());
    }

    #[test]
    fn invalid_names_are_rejected() {
        assert_eq!(Key::parse("").err(), Some(NameError::Empty));
        assert_eq!(
            Key::parse(&"x".repeat(MAX_NAME_LENGTH + 1)).err(),
            Some(NameError::TooLong {
                length: MAX_NAME_LENGTH + 1
            })
        );
        assert_eq!(
            Key::parse("user 42").err(),
            Some(NameError::InvalidCharacter {
                character: ' ',
                index: 4
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
