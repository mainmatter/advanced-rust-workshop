//! # Exercise
//!
//! Look at what `insert` actually does with the references you hand it:
//!
//! ```text
//! pub fn insert(&mut self, bucket: &Bucket, key: &Key, value: Value) -> Option<Value> {
//!     self.buckets
//!         .entry(bucket.clone())
//!         .or_default()
//!         .insert(key.clone(), value)
//! }
//! ```
//!
//! It borrows, and then immediately clones, because a map has to own its keys. The signature says
//! "lend me these" and the body says "actually, mine now". Two allocations per insert that the
//! caller cannot see, cannot avoid, and is not told about.
//!
//! `HashMap` does not do this. `insert(K, V)` takes ownership because it needs ownership, and
//! `get(&Q)` borrows because it does not. The asymmetry is the API telling you the truth.
//!
//! Make `minidb` tell the truth too:
//!
//! ```text
//! Store::insert(&mut self, bucket: Bucket, key: Key, value: Value) -> Option<Value>
//! ```
//!
//! and delete the two `clone` calls the body no longer needs. Leave `get` and `remove` taking
//! shared references: they genuinely only need to look, and a caller who still owns its names can
//! keep them.
//!
//! Afterwards the cost is where the caller can see it:
//!
//! ```compile_fail,E0382
//! use borrowing_ownership::{Bucket, Key, Store, Value};
//!
//! let mut store = Store::new();
//! let users = Bucket::parse("users").unwrap();
//! let id = Key::parse("42").unwrap();
//!
//! store.insert(users, id, Value::new("Alice"));
//!
//! println!("{}", id.as_str());
//! ```
//!
//! This exercise starts out **not compiling**: the tests are written against the new signatures.

use std::{
    collections::HashMap,
    fmt::{self, Debug, Formatter},
};

const MAX_NAME_LENGTH: usize = 64;

/// An in-memory key-value store, partitioned into named buckets.
pub struct Store {
    buckets: HashMap<Bucket, HashMap<Key, Value>>,
}

impl Store {
    /// Creates an empty store.
    pub fn new() -> Self {
        Self {
            buckets: HashMap::new(),
        }
    }

    /// Inserts a value, returning the value it replaced, if any.
    pub fn insert(&mut self, bucket: Bucket, key: Key, value: Value) -> Option<Value> {
        self.buckets.entry(bucket).or_default().insert(key, value)
    }

    /// Looks up a value.
    pub fn get(&self, bucket: &Bucket, key: &Key) -> Option<&Value> {
        self.buckets.get(bucket)?.get(key)
    }

    /// Removes a value, returning it if it was there.
    pub fn remove(&mut self, bucket: &Bucket, key: &Key) -> Option<Value> {
        self.buckets.get_mut(bucket)?.remove(key)
    }

    /// Lists the buckets, including any that have been emptied.
    pub fn buckets(&self) -> impl Iterator<Item = &Bucket> {
        self.buckets.keys()
    }

    /// Keeps only the values the predicate accepts, dropping any bucket left empty.
    pub fn retain<F>(&mut self, mut predicate: F)
    where
        F: FnMut(&Bucket, &Key, &Value) -> bool,
    {
        self.buckets.retain(|bucket, values| {
            values.retain(|key, value| predicate(bucket, key, value));
            !values.is_empty()
        });
    }
}

/// The name of a bucket.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

impl TryFrom<&str> for Key {
    type Error = NameError;

    fn try_from(raw: &str) -> Result<Self, Self::Error> {
        Self::parse(raw)
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
    fn the_store_takes_the_names_it_keeps() {
        let mut store = Store::new();
        let users = Bucket::parse("users").unwrap();
        let id = Key::parse("42").unwrap();

        let replaced = store.insert(users.clone(), id.clone(), Value::new("Alice"));

        assert!(replaced.is_none());
        assert_eq!(store.get(&users, &id).map(Value::as_str), Some("Alice"));
    }

    #[test]
    fn looking_up_still_only_borrows() {
        let mut store = Store::new();
        let users = Bucket::parse("users").unwrap();
        let id = Key::parse("42").unwrap();
        store.insert(users.clone(), id.clone(), Value::new("Alice"));

        assert_eq!(store.get(&users, &id).map(Value::as_str), Some("Alice"));
        assert_eq!(
            store.remove(&users, &id).map(Value::into_inner).as_deref(),
            Some("Alice")
        );
        assert_eq!(store.get(&users, &id).map(Value::as_str), None);
    }
}
