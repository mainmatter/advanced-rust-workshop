//! # Exercise
//!
//! A type called `Key` that cannot be a key in a `HashMap` is a joke at your own expense. Fix it, and
//! then collect the winnings.
//!
//! 1. Derive `PartialEq`, `Eq`, `Hash` and `Clone` on `Bucket` and `Key`. Derive all four together and
//!    never hand-write only one of `Hash` and `PartialEq`: the contract is that equal values hash
//!    equally, and the derives are the only way to keep that true for free.
//! 2. Now that the newtypes can be map keys, store them as map keys: `Store` should hold a
//!    `HashMap<Bucket, HashMap<Key, Value>>`. All the `as_str().to_owned()` juggling in `insert`,
//!    `get` and `remove` goes away.
//! 3. Add `Store::buckets(&self) -> impl Iterator<Item = &Bucket>`. This one only compiles once step 2
//!    is done, which is the point: a store keyed by `String` cannot hand out a `&Bucket` it does not
//!    have.
//! 4. Add `impl TryFrom<&str> for Key`, delegating to `Key::parse`. Same check, same error, second
//!    door, and this one is the door generic code knocks on.
//!
//! This exercise starts out **not compiling**: the tests are written against all four.

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
        write!(f, "Value(<redacted, {} bytes>)", self.0.len())
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
    use crate::{Bucket, Key, NameError, Store, Value};
    use std::collections::HashMap;

    #[test]
    fn keys_are_map_keys() {
        let mut map = HashMap::new();
        map.insert(Key::parse("users/42").unwrap(), "Alice");

        assert_eq!(map[&Key::parse("users/42").unwrap()], "Alice");
    }

    #[test]
    fn equal_keys_are_the_same_entry() {
        let mut map = HashMap::new();
        map.insert(Key::parse("users/42").unwrap(), "Alice");
        map.insert(Key::parse("users/42").unwrap(), "Bob");

        assert_eq!(map.len(), 1);
    }

    #[test]
    fn buckets_are_handed_out_as_buckets() {
        let mut store = Store::new();
        let users = Bucket::parse("users").unwrap();
        store.insert(&users, &Key::parse("42").unwrap(), Value::new("Alice"));

        let names = store.buckets().map(Bucket::as_str).collect::<Vec<_>>();

        assert_eq!(names, ["users"]);
    }

    #[test]
    fn try_from_is_parse_by_another_name() {
        assert_eq!(
            Key::try_from("users/42").unwrap().as_str(),
            Key::parse("users/42").unwrap().as_str()
        );
        assert_eq!(
            Key::try_from("user 42").unwrap_err(),
            NameError::InvalidCharacter {
                character: ' ',
                index: 4
            }
        );
    }

    #[test]
    fn the_store_still_works() {
        let mut store = Store::new();
        let users = Bucket::parse("users").unwrap();
        let id = Key::parse("42").unwrap();

        store.insert(&users, &id, Value::new("Alice"));

        assert_eq!(store.get(&users, &id).map(Value::as_str), Some("Alice"));
        assert_eq!(
            store.remove(&users, &id).map(Value::into_inner).as_deref(),
            Some("Alice")
        );
    }
}
