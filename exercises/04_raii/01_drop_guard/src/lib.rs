//! # Exercise
//!
//! An abandoned transaction should not quietly become permanent. Make the safe outcome the automatic
//! one: implement `Drop` for `Transaction`, so a transaction that is neither committed nor rolled back
//! undoes itself.
//!
//! Two things will get in your way, and both are the lesson.
//!
//! `Drop::drop` hands you `&mut self`, not `self`, so you cannot move the undo log out of it: the
//! value still has to be dropped after you are done with it. `mem::take` swaps in an empty `Vec` and
//! gives you the full one, which is the usual way out. `Option::take` is the same trick for a single
//! value.
//!
//! `commit` consumes `self`, so a committed transaction is dropped the instant `commit` returns and
//! your new `Drop` impl runs immediately afterwards. Make sure it does not undo the work that was just
//! committed: `Drop` has to be able to tell that a decision was already made.

use std::collections::HashMap;
use std::fmt::{self, Debug, Formatter};
use std::mem;

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

    /// Starts a transaction, borrowing the store until it finishes.
    pub fn begin(&mut self) -> Transaction<'_> {
        Transaction {
            store: self,
            undo: Vec::new(),
        }
    }

    /// Inserts a value, returning the value it replaced, if any.
    pub fn insert(&mut self, bucket: &Bucket, key: &Key, value: Value) -> Option<Value> {
        self.buckets
            .entry(bucket.clone())
            .or_default()
            .insert(key.clone(), value)
    }

    /// Looks up a value.
    pub fn get(&self, bucket: &Bucket, key: &Key) -> Option<&Value> {
        self.buckets.get(bucket)?.get(key)
    }

    /// Removes a value, returning it if it was there.
    pub fn remove(&mut self, bucket: &Bucket, key: &Key) -> Option<Value> {
        self.buckets.get_mut(bucket)?.remove(key)
    }

    /// Lists the buckets that hold at least one value.
    pub fn buckets(&self) -> impl Iterator<Item = &Bucket> {
        self.buckets.keys()
    }
}

/// A set of changes applied to a [`Store`] together.
///
/// Changes take effect immediately. Until [`Transaction::commit`] is called, every one of them can
/// still be taken back by [`Transaction::rollback`].
pub struct Transaction<'store> {
    store: &'store mut Store,
    undo: Vec<Undo>,
}

impl Transaction<'_> {
    /// Inserts a value as part of this transaction.
    pub fn insert(&mut self, bucket: &Bucket, key: &Key, value: Value) {
        let previous = self.store.insert(bucket, key, value);
        self.undo.push(Undo {
            bucket: bucket.clone(),
            key: key.clone(),
            previous,
        });
    }

    /// Removes a value as part of this transaction.
    pub fn remove(&mut self, bucket: &Bucket, key: &Key) {
        let previous = self.store.remove(bucket, key);
        self.undo.push(Undo {
            bucket: bucket.clone(),
            key: key.clone(),
            previous,
        });
    }

    /// Keeps every change made through this transaction.
    pub fn commit(self) {}

    /// Takes back every change made through this transaction.
    pub fn rollback(mut self) {
        self.undo_everything();
    }

    fn undo_everything(&mut self) {
        for undo in mem::take(&mut self.undo).into_iter().rev() {
            match undo.previous {
                Some(value) => self.store.insert(&undo.bucket, &undo.key, value),
                None => self.store.remove(&undo.bucket, &undo.key),
            };
        }
    }
}

impl Drop for Transaction<'_> {
    fn drop(&mut self) {
        todo!()
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
    use crate::{Bucket, Key, Store, Value};

    #[test]
    fn an_abandoned_transaction_undoes_itself() {
        let mut store = Store::new();
        let users = Bucket::parse("users").unwrap();
        let id = Key::parse("42").unwrap();

        {
            let mut txn = store.begin();
            txn.insert(&users, &id, Value::new("Alice"));
        }

        assert_eq!(store.get(&users, &id).map(Value::as_str), None);
    }

    #[test]
    fn a_committed_transaction_survives_being_dropped() {
        let mut store = Store::new();
        let users = Bucket::parse("users").unwrap();
        let id = Key::parse("42").unwrap();

        let mut txn = store.begin();
        txn.insert(&users, &id, Value::new("Alice"));
        txn.commit();

        assert_eq!(store.get(&users, &id).map(Value::as_str), Some("Alice"));
    }

    #[test]
    fn an_explicit_rollback_still_works() {
        let mut store = Store::new();
        let users = Bucket::parse("users").unwrap();
        let id = Key::parse("42").unwrap();

        let mut txn = store.begin();
        txn.insert(&users, &id, Value::new("Alice"));
        txn.rollback();

        assert_eq!(store.get(&users, &id).map(Value::as_str), None);
    }

    #[test]
    fn undoing_restores_what_was_there_before() {
        let mut store = Store::new();
        let users = Bucket::parse("users").unwrap();
        let id = Key::parse("42").unwrap();
        store.insert(&users, &id, Value::new("Alice"));

        {
            let mut txn = store.begin();
            txn.insert(&users, &id, Value::new("Bob"));
            txn.insert(&users, &id, Value::new("Carol"));
            txn.remove(&users, &id);
        }

        assert_eq!(store.get(&users, &id).map(Value::as_str), Some("Alice"));
    }

    #[test]
    fn bailing_out_halfway_undoes_the_first_half() {
        let mut store = Store::new();
        let users = Bucket::parse("users").unwrap();
        let alice = Key::parse("42").unwrap();
        let bob = Key::parse("43").unwrap();

        let outcome = write_both(&mut store, &users, &alice, &bob);

        assert!(outcome.is_err());
        assert_eq!(store.get(&users, &alice).map(Value::as_str), None);
        assert_eq!(store.get(&users, &bob).map(Value::as_str), None);
    }

    fn write_both(store: &mut Store, bucket: &Bucket, first: &Key, second: &Key) -> Result<(), ()> {
        let mut txn = store.begin();
        txn.insert(bucket, first, Value::new("Alice"));

        let second_value = fetch_the_other_value()?;

        txn.insert(bucket, second, second_value);
        txn.commit();

        Ok(())
    }

    fn fetch_the_other_value() -> Result<Value, ()> {
        Err(())
    }
}
