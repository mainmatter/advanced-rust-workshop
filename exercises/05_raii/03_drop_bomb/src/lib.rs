//! # Exercise
//!
//! Rolling back is the safe outcome, and it is now automatic. It is also silent: a transaction
//! somebody forgot to commit behaves exactly like one they meant to abandon, and the bug survives
//! into production disguised as a design decision.
//!
//! Arm the guard. Turn `Transaction` into a **drop bomb**: dropping one that was neither committed
//! nor rolled back still undoes the work, and then panics, because reaching that line is a bug in
//! the calling code.
//!
//! The struct already carries a `finished` flag for you. Make `commit` and `rollback` set it, and
//! make `drop` react to it.
//!
//! One rule you must not skip. A panic while another panic is unwinding aborts the process
//! immediately: no unwinding, no test failure, no message, just a dead process. So a drop bomb has
//! to ask first:
//!
//! ```text
//! if !thread::panicking() {
//!     panic!("...");
//! }
//! ```
//!
//! The last test proves it. If you get this wrong it will not fail, it will take the whole test
//! binary down with it, which is exactly how this bug presents itself in real code.

use std::{
    collections::HashMap,
    fmt::{self, Debug, Formatter},
    mem, thread,
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

    /// Starts a transaction, borrowing the store until it finishes.
    pub fn begin(&mut self) -> Transaction<'_> {
        Transaction {
            store: self,
            undo: Vec::new(),
            finished: false,
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
pub struct Transaction<'store> {
    store: &'store mut Store,
    undo: Vec<Undo>,
    finished: bool,
}

impl Transaction<'_> {
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

    /// Keeps every change made through this transaction.
    pub fn commit(mut self) {
        self.undo.clear();
    }

    /// Takes back every change made through this transaction.
    pub fn rollback(mut self) {
        self.undo_everything();
    }

    fn undo_everything(&mut self) {
        for undo in mem::take(&mut self.undo).into_iter().rev() {
            match undo.previous {
                Some(value) => self.store.insert(undo.bucket, undo.key, value),
                None => self.store.remove(&undo.bucket, &undo.key),
            };
        }
    }
}

impl Drop for Transaction<'_> {
    fn drop(&mut self) {
        self.undo_everything();
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
    #[should_panic(expected = "neither committed nor rolled back")]
    fn forgetting_to_decide_is_a_bug_and_says_so() {
        let mut store = Store::new();
        let users = Bucket::parse("users").unwrap();
        let id = Key::parse("42").unwrap();

        let mut tx = store.begin();
        tx.insert(users.clone(), id.clone(), Value::new("Alice"));
    }

    #[test]
    fn committing_is_a_decision() {
        let mut store = Store::new();
        let users = Bucket::parse("users").unwrap();
        let id = Key::parse("42").unwrap();

        let mut tx = store.begin();
        tx.insert(users.clone(), id.clone(), Value::new("Alice"));
        tx.commit();

        assert_eq!(store.get(&users, &id).map(Value::as_str), Some("Alice"));
    }

    #[test]
    fn rolling_back_is_a_decision() {
        let mut store = Store::new();
        let users = Bucket::parse("users").unwrap();
        let id = Key::parse("42").unwrap();

        let mut tx = store.begin();
        tx.insert(users.clone(), id.clone(), Value::new("Alice"));
        tx.rollback();

        assert_eq!(store.get(&users, &id).map(Value::as_str), None);
    }

    #[test]
    fn the_bomb_still_undoes_the_work_on_its_way_out() {
        let mut store = Store::new();
        let users = Bucket::parse("users").unwrap();
        let id = Key::parse("42").unwrap();

        let outcome = panic::catch_unwind(AssertUnwindSafe(|| {
            let mut tx = store.begin();
            tx.insert(users.clone(), id.clone(), Value::new("Alice"));
        }));

        assert!(outcome.is_err());
        assert_eq!(store.get(&users, &id).map(Value::as_str), None);
    }

    #[test]
    fn a_panic_in_flight_is_not_replaced_by_the_bomb() {
        let mut store = Store::new();
        let users = Bucket::parse("users").unwrap();
        let id = Key::parse("42").unwrap();

        let outcome = panic::catch_unwind(AssertUnwindSafe(|| {
            let mut tx = store.begin();
            tx.insert(users.clone(), id.clone(), Value::new("Alice"));
            panic!("the original problem");
        }));

        let message = *outcome.unwrap_err().downcast::<&str>().unwrap();

        assert_eq!(message, "the original problem");
        assert_eq!(store.get(&users, &id).map(Value::as_str), None);
    }
}
