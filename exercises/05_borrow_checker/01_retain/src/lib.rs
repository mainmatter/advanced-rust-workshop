//! # Exercise
//!
//! Add a bulk delete:
//!
//! ```text
//! Store::retain<F>(&mut self, predicate: F)
//! where
//!     F: FnMut(&Bucket, &Key, &Value) -> bool,
//! ```
//!
//! Every value the predicate rejects is removed, and a bucket left holding nothing disappears with
//! it, so it stops showing up in `buckets()`.
//!
//! Write the obvious version first: walk the buckets, walk the values, remove the ones that fail.
//! The compiler will stop you with `E0502`, and it is right to. You are asking to hold a reference
//! into a collection while changing the shape of that same collection, which is how iterator
//! invalidation hands you a dangling pointer in languages that allow it.
//!
//! There are three honest ways out, and all three are worth knowing:
//!
//! - collect what you want to delete into a `Vec` first, then delete in a second pass. The general
//!   answer, and it costs an allocation.
//! - `HashMap::retain`, which does the whole job in one pass because it was written from the inside,
//!   where the borrow is not a problem.
//! - restructure so the mutation happens through the iterator itself, which for a map means `retain`
//!   or `drain`, and for a `Vec` also `retain_mut`.
//!
//! Any of them passes the tests.

use std::collections::HashMap;
use std::fmt::{self, Debug, Formatter};
use std::mem;
use std::thread;

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

    /// Runs `changes` in a transaction, committing it if they succeed and rolling it back if they do
    /// not.
    pub fn transaction<F, T, E>(&mut self, changes: F) -> Result<T, E>
    where
        F: FnOnce(&mut Transaction<'_>) -> Result<T, E>,
    {
        let mut tx = self.begin();

        match changes(&mut tx) {
            Ok(value) => {
                tx.commit();
                Ok(value)
            }
            Err(error) => {
                tx.rollback();
                Err(error)
            }
        }
    }

    /// Starts a transaction, borrowing the store until it finishes.
    pub fn begin(&mut self) -> Transaction<'_> {
        Transaction {
            store: self,
            undo: Vec::new(),
            finished: false,
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

    /// Lists the buckets, including any that have been emptied.
    pub fn buckets(&self) -> impl Iterator<Item = &Bucket> {
        self.buckets.keys()
    }

    /// Keeps only the values the predicate accepts, dropping any bucket left empty.
    pub fn retain<F>(&mut self, predicate: F)
    where
        F: FnMut(&Bucket, &Key, &Value) -> bool,
    {
        todo!()
    }
}

/// A set of changes applied to a [`Store`] together.
///
/// Changes take effect immediately. Until [`Transaction::commit`] is called, every one of them can
/// still be taken back by [`Transaction::rollback`].
pub struct Transaction<'store> {
    store: &'store mut Store,
    undo: Vec<Undo>,
    finished: bool,
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
    pub fn commit(mut self) {
        self.finished = true;
        self.undo.clear();
    }

    /// Takes back every change made through this transaction.
    pub fn rollback(mut self) {
        self.finished = true;
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
        if self.finished {
            return;
        }

        self.undo_everything();

        if !thread::panicking() {
            panic!("transaction dropped while neither committed nor rolled back");
        }
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
    fn rejected_values_are_removed_and_the_rest_stay() {
        let mut store = Store::new();
        let users = Bucket::parse("users").unwrap();
        let alice = Key::parse("42").unwrap();
        let bob = Key::parse("43").unwrap();
        store.insert(&users, &alice, Value::new("Alice"));
        store.insert(&users, &bob, Value::new("Bob"));

        store.retain(|_, _, value| value.as_str().starts_with('A'));

        assert_eq!(store.get(&users, &alice).map(Value::as_str), Some("Alice"));
        assert_eq!(store.get(&users, &bob).map(Value::as_str), None);
    }

    #[test]
    fn emptied_buckets_disappear() {
        let mut store = Store::new();
        let users = Bucket::parse("users").unwrap();
        let orders = Bucket::parse("orders").unwrap();
        let id = Key::parse("42").unwrap();
        store.insert(&users, &id, Value::new("Alice"));
        store.insert(&orders, &id, Value::new("a book"));

        store.retain(|bucket, _, _| bucket.as_str() == "users");

        let remaining = store.buckets().map(Bucket::as_str).collect::<Vec<_>>();

        assert_eq!(remaining, ["users"]);
    }

    #[test]
    fn the_predicate_sees_every_value_once() {
        let mut store = Store::new();
        let users = Bucket::parse("users").unwrap();
        let orders = Bucket::parse("orders").unwrap();
        store.insert(&users, &Key::parse("42").unwrap(), Value::new("Alice"));
        store.insert(&users, &Key::parse("43").unwrap(), Value::new("Bob"));
        store.insert(&orders, &Key::parse("1").unwrap(), Value::new("a book"));

        let mut seen = Vec::new();
        store.retain(|bucket, key, _| {
            seen.push(format!("{}/{}", bucket.as_str(), key.as_str()));
            true
        });
        seen.sort();

        assert_eq!(seen, ["orders/1", "users/42", "users/43"]);
    }

    #[test]
    fn keeping_everything_changes_nothing() {
        let mut store = Store::new();
        let users = Bucket::parse("users").unwrap();
        let id = Key::parse("42").unwrap();
        store.insert(&users, &id, Value::new("Alice"));

        store.retain(|_, _, _| true);

        assert_eq!(store.get(&users, &id).map(Value::as_str), Some("Alice"));
        assert_eq!(store.buckets().count(), 1);
    }
}
