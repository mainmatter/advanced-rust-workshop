//! # Exercise
//!
//! `Bucket` and `Key` now derive `Debug`. That is one line each, it is the right default, and there is
//! nothing to think about: a bucket name is not a secret, and a programmer staring at a panic message
//! wants to see it.
//!
//! `Value` is different. It holds whatever the user of `minidb` put in, which in the field means
//! session tokens, email addresses and the occasional password. `Debug` is not a debugging aid for a
//! type like this, it is the format your data arrives in when someone logs a struct that happens to
//! contain one, or when a `.unwrap()` fails at three in the morning.
//!
//! So `Value` gets a hand-written `Debug` that tells you what you need in order to debug and nothing
//! you would regret:
//!
//! ```text
//! Value(<redacted, 7 bytes>)
//! ```
//!
//! Implement it. The length is deliberate: it is enough to tell an empty value from a truncated one,
//! and useless to anyone reading your logs.

use std::collections::HashMap;
use std::fmt::{self, Debug, Formatter};

const MAX_NAME_LENGTH: usize = 64;

/// An in-memory key-value store, partitioned into named buckets.
pub struct Store {
    buckets: HashMap<String, HashMap<String, Value>>,
}

impl Store {
    /// Creates an empty store.
    pub fn new() -> Self {
        Self {
            buckets: HashMap::new(),
        }
    }

    /// Inserts a value, returning the value it replaced, if any.
    pub fn insert(&mut self, bucket: &Bucket, key: &Key, value: Value) -> Option<Value> {
        self.buckets
            .entry(bucket.as_str().to_owned())
            .or_default()
            .insert(key.as_str().to_owned(), value)
    }

    /// Looks up a value.
    pub fn get(&self, bucket: &Bucket, key: &Key) -> Option<&Value> {
        self.buckets.get(bucket.as_str())?.get(key.as_str())
    }

    /// Removes a value, returning it if it was there.
    pub fn remove(&mut self, bucket: &Bucket, key: &Key) -> Option<Value> {
        self.buckets.get_mut(bucket.as_str())?.remove(key.as_str())
    }
}

/// The name of a bucket.
#[derive(Debug)]
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
#[derive(Debug)]
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

/// A value held in the store.
pub struct Value(String);

impl Value {
    /// Wraps a value. Any bytes will do: `minidb` does not look inside.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Borrows the value.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes the wrapper, returning the value.
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl Debug for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
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
    use crate::{Bucket, Key, Store, Value};

    #[test]
    fn names_are_shown_in_full() {
        let bucket = Bucket::parse("users").unwrap();
        let key = Key::parse("users/42").unwrap();

        assert_eq!(format!("{bucket:?}"), r#"Bucket("users")"#);
        assert_eq!(format!("{key:?}"), r#"Key("users/42")"#);
    }

    #[test]
    fn values_are_redacted() {
        let value = Value::new("hunter2");

        assert_eq!(format!("{value:?}"), "Value(<redacted, 7 bytes>)");
    }

    #[test]
    fn redaction_survives_being_nested_in_something_else() {
        let mut store = Store::new();
        let users = Bucket::parse("users").unwrap();
        let id = Key::parse("42").unwrap();
        store.insert(&users, &id, Value::new("s3cret token"));

        let rendered = format!("{:?}", store.get(&users, &id));

        assert!(!rendered.contains("s3cret"));
        assert!(rendered.contains("12 bytes"));
    }
}
