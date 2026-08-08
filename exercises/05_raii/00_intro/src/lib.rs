//! # Exercise
//!
//! Nothing to write here. Read the four tests, run `wr`, and move on.
//!
//! `minidb` has transactions. A `Transaction` applies each change to the store as it goes and
//! records how to undo it, so `rollback` can put everything back, and `commit` keeps the changes.
//!
//! Building one needs the transaction to reach the store somehow, and there is exactly one way to
//! do that with what you have met so far: give it the store.
//!
//! ```text
//! Store::begin(self) -> Transaction
//! Transaction::commit(self) -> Store
//! Transaction::rollback(self) -> Store
//! ```
//!
//! It works. The four tests below pass, and they also spell out why nobody would ship it:
//!
//! - the store is *inside* the transaction, so `Transaction` has to grow its own copy of every
//!   `Store` method a caller might want. `get` is the first one, and it would not be the last;
//! - every call site has to catch the store on the way out: `let store = tx.commit();`;
//! - `commit` and `rollback` both return a `Store`, so the return type says nothing about
//!   committing or rolling back. It is bookkeeping;
//! - an early return does not leave half the work behind, it loses the whole store, because the
//!   store is inside the value that just went out of scope.
//!
//! What we want is for the store to stay where it is and lend itself to the transaction for a
//! while. Lending exclusively is what `&mut` is for, and a struct that keeps one needs a lifetime.
//! That is the next exercise.
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

    /// Hands the store to a transaction, which gives it back when it finishes.
    pub fn begin(self) -> Transaction {
        Transaction {
            store: self,
            undo: Vec::new(),
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

/// A set of changes applied to a [`Store`] together.
///
/// Changes take effect immediately. Until [`Transaction::commit`] is called, every one of them can
/// still be taken back by [`Transaction::rollback`].
pub struct Transaction {
    store: Store,
    undo: Vec<Undo>,
}

impl Transaction {
    /// Inserts a value as part of this transaction.
    pub fn insert(&mut self, bucket: Bucket, key: Key, value: Value) {
        let previous = self.store.insert(bucket.clone(), key.clone(), value);
        self.undo.push(Undo {
            bucket,
            key,
            previous,
        });
    }

    /// Removes a value as part of this transaction.
    pub fn remove(&mut self, bucket: Bucket, key: Key) {
        let previous = self.store.remove(&bucket, &key);
        self.undo.push(Undo {
            bucket,
            key,
            previous,
        });
    }

    /// Looks up a value. The store is inside the transaction, so this is the only way to read it.
    pub fn get(&self, bucket: &Bucket, key: &Key) -> Option<&Value> {
        self.store.get(bucket, key)
    }

    /// Keeps every change made through this transaction, and gives the store back.
    pub fn commit(self) -> Store {
        self.store
    }

    /// Takes back every change made through this transaction, and gives the store back.
    pub fn rollback(mut self) -> Store {
        for undo in self.undo.into_iter().rev() {
            match undo.previous {
                Some(value) => self.store.insert(undo.bucket, undo.key, value),
                None => self.store.remove(&undo.bucket, &undo.key),
            };
        }

        self.store
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

struct Undo {
    bucket: Bucket,
    key: Key,
    previous: Option<Value>,
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
    fn the_store_has_to_be_handed_back() {
        let store = Store::new();
        let users = Bucket::parse("users").unwrap();
        let id = Key::parse("42").unwrap();

        let mut tx = store.begin();
        tx.insert(users.clone(), id.clone(), Value::new("Alice"));
        let store = tx.commit();

        assert_eq!(store.get(&users, &id).map(Value::as_str), Some("Alice"));
    }

    #[test]
    fn a_rollback_hands_it_back_too() {
        let mut store = Store::new();
        let users = Bucket::parse("users").unwrap();
        let id = Key::parse("42").unwrap();
        store.insert(users.clone(), id.clone(), Value::new("Alice"));

        let mut tx = store.begin();
        tx.insert(users.clone(), id.clone(), Value::new("Bob"));
        let store = tx.rollback();

        assert_eq!(store.get(&users, &id).map(Value::as_str), Some("Alice"));
    }

    #[test]
    fn every_read_has_to_go_through_the_transaction() {
        let store = Store::new();
        let users = Bucket::parse("users").unwrap();
        let id = Key::parse("42").unwrap();

        let mut tx = store.begin();
        tx.insert(users.clone(), id.clone(), Value::new("Alice"));

        assert_eq!(tx.get(&users, &id).map(Value::as_str), Some("Alice"));

        tx.commit();
    }

    #[test]
    fn bailing_out_halfway_loses_the_store_entirely() {
        let store = Store::new();
        let users = Bucket::parse("users").unwrap();
        let alice = Key::parse("42").unwrap();
        let bob = Key::parse("43").unwrap();

        let outcome = write_both(store, &users, &alice, &bob);

        assert!(outcome.is_err());
    }

    fn write_both(store: Store, bucket: &Bucket, first: &Key, second: &Key) -> Result<Store, ()> {
        let mut tx = store.begin();
        tx.insert(bucket.clone(), first.clone(), Value::new("Alice"));

        let second_value = fetch_the_other_value()?;

        tx.insert(bucket.clone(), second.clone(), second_value);

        Ok(tx.commit())
    }

    fn fetch_the_other_value() -> Result<Value, ()> {
        Err(())
    }
}
