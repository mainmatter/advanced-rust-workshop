//! # Exercise
//!
//! The drop bomb catches the mistake at runtime, in the crash you were hoping to avoid. It is a
//! smoke detector: useful, and no substitute for not setting the kitchen on fire.
//!
//! So stop asking callers to remember. Add a closure API that makes the decision for them:
//!
//! ```text
//! Store::transaction<F, T, E>(&mut self, changes: F) -> Result<T, E>
//! where
//!     F: FnOnce(&mut Transaction<'_>) -> Result<T, E>,
//! ```
//!
//! It begins a transaction, hands it to the closure, and then commits if the closure returned `Ok`
//! and rolls back if it returned `Err`, passing the value or the error straight back to the caller.
//!
//! Note what happens to the bomb: it can no longer go off through this path, because the only code
//! that could forget is code you just wrote once. Keep it armed anyway, for the callers who still
//! reach for `begin`.
//!
//! This exercise starts out **not compiling**: the tests are written against the method you are about
//! to add.

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
    use std::panic::{self, AssertUnwindSafe};

    #[test]
    fn returning_ok_commits_and_hands_the_value_back() {
        let mut store = Store::new();
        let users = Bucket::parse("users").unwrap();
        let id = Key::parse("42").unwrap();

        let outcome = store.transaction(|txn| {
            txn.insert(&users, &id, Value::new("Alice"));
            Ok::<_, Rejected>(7)
        });

        assert_eq!(outcome, Ok(7));
        assert_eq!(store.get(&users, &id).map(Value::as_str), Some("Alice"));
    }

    #[test]
    fn returning_err_rolls_back_and_hands_the_error_back() {
        let mut store = Store::new();
        let users = Bucket::parse("users").unwrap();
        let alice = Key::parse("42").unwrap();
        let bob = Key::parse("43").unwrap();

        let outcome = store.transaction(|txn| {
            txn.insert(&users, &alice, Value::new("Alice"));
            txn.insert(&users, &bob, Value::new("Bob"));
            Err::<(), _>(Rejected)
        });

        assert_eq!(outcome, Err(Rejected));
        assert_eq!(store.get(&users, &alice).map(Value::as_str), None);
        assert_eq!(store.get(&users, &bob).map(Value::as_str), None);
    }

    #[test]
    fn the_question_mark_operator_works_inside() {
        let mut store = Store::new();
        let users = Bucket::parse("users").unwrap();
        let id = Key::parse("42").unwrap();

        let outcome = store.transaction(|txn| {
            txn.insert(&users, &id, Value::new("Alice"));
            let value = rejected()?;
            txn.insert(&users, &id, value);
            Ok(())
        });

        assert_eq!(outcome, Err(Rejected));
        assert_eq!(store.get(&users, &id).map(Value::as_str), None);
    }

    #[test]
    fn a_panic_inside_still_rolls_back_and_still_panics_once() {
        let mut store = Store::new();
        let users = Bucket::parse("users").unwrap();
        let id = Key::parse("42").unwrap();

        let outcome = panic::catch_unwind(AssertUnwindSafe(|| {
            store.transaction::<_, (), Rejected>(|txn| {
                txn.insert(&users, &id, Value::new("Alice"));
                panic!("the original problem");
            })
        }));

        let message = *outcome.unwrap_err().downcast::<&str>().unwrap();

        assert_eq!(message, "the original problem");
        assert_eq!(store.get(&users, &id).map(Value::as_str), None);
    }

    #[test]
    #[should_panic(expected = "neither committed nor rolled back")]
    fn begin_is_still_there_and_still_armed() {
        let mut store = Store::new();
        let users = Bucket::parse("users").unwrap();
        let id = Key::parse("42").unwrap();

        let mut txn = store.begin();
        txn.insert(&users, &id, Value::new("Alice"));
    }

    #[derive(Debug, PartialEq, Eq)]
    struct Rejected;

    fn rejected() -> Result<Value, Rejected> {
        Err(Rejected)
    }
}
