//! # Exercise
//!
//! Nothing to write here. Read the doctest, run `wr`, and move on.
//!
//! Every ergonomic wish in `minidb` so far has been satisfiable by adding a method to a type we own.
//! That runs out the moment the type belongs to somebody else.
//!
//! Rust's **orphan rule** says you may implement a trait for a type only if you own the trait or you
//! own the type. Both foreign is forbidden, and the reason is coherence: if two crates could both
//! `impl Display for Vec<u8>`, a program depending on both would have two answers and no way to
//! choose.
//!
//! ```compile_fail,E0117
//! use std::fmt::{self, Display, Formatter};
//!
//! impl Display for Option<String> {
//!     fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
//!         todo!()
//!     }
//! }
//! ```
//!
//! So you cannot add a method to `str`, or to `Option`, or to anything else you did not define. What
//! you *can* do is define your own trait, which you own, and implement it for their type. That is an
//! **extension trait**, and it is the whole content of this chapter.
//!
//! The test below shows the ergonomics we are about to fix.

use std::collections::HashMap;
use std::fmt::{self, Debug, Formatter};
use std::marker::PhantomData;
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
        F: FnOnce(&mut Transaction<'_, ReadWrite>) -> Result<T, E>,
    {
        let mut txn = self.begin();

        match changes(&mut txn) {
            Ok(value) => {
                txn.commit();
                Ok(value)
            }
            Err(error) => {
                txn.rollback();
                Err(error)
            }
        }
    }

    /// Starts a transaction, borrowing the store until it finishes.
    pub fn begin(&mut self) -> Transaction<'_, ReadWrite> {
        Transaction {
            store: self,
            undo: Vec::new(),
            finished: false,
            _state: PhantomData,
        }
    }

    /// Starts a transaction that is only allowed to read.
    pub fn begin_read(&mut self) -> Transaction<'_, ReadOnly> {
        Transaction {
            store: self,
            undo: Vec::new(),
            finished: true,
            _state: PhantomData,
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

    /// Renders the whole store as text, with buckets and keys in sorted order.
    pub fn export(&self) -> String {
        let mut buckets = self.buckets.iter().collect::<Vec<_>>();
        buckets.sort_by(|(left, _), (right, _)| left.cmp(right));

        let mut writer = Writer::new();

        for (bucket, values) in buckets {
            let mut entries = values.iter().collect::<Vec<_>>();
            entries.sort_by(|(left, _), (right, _)| left.cmp(right));

            let mut open = writer.bucket(bucket);

            for (key, value) in entries {
                open = open.entry(key, value);
            }

            writer = open.end();
        }

        writer.finish()
    }
}

/// A set of changes applied to a [`Store`] together.
///
/// Changes take effect immediately. Until [`Transaction::commit`] is called, every one of them can
/// still be taken back by [`Transaction::rollback`].
pub struct Transaction<'store, S>
where
    S: State,
{
    store: &'store mut Store,
    undo: Vec<Undo>,
    finished: bool,
    _state: PhantomData<S>,
}

impl<S> Transaction<'_, S>
where
    S: State,
{
    /// Looks up a value as part of this transaction.
    pub fn get(&self, bucket: &Bucket, key: &Key) -> Option<&Value> {
        self.store.get(bucket, key)
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

impl Transaction<'_, ReadWrite> {
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
        self.finished = true;
        self.undo.clear();
    }

    /// Takes back every change made through this transaction.
    pub fn rollback(mut self) {
        self.finished = true;
        self.undo_everything();
    }
}

impl<S> Drop for Transaction<'_, S>
where
    S: State,
{
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

/// What a [`Transaction`] is allowed to do.
pub trait State {}

/// A transaction that may only read.
pub struct ReadOnly;

impl State for ReadOnly {}

/// A transaction that may read and write.
pub struct ReadWrite;

impl State for ReadWrite {}

/// Renders a [`Store`] as text, one bucket at a time.
pub struct Writer<S>
where
    S: Section,
{
    output: String,
    _section: PhantomData<S>,
}

impl Writer<Root> {
    /// Starts an empty document.
    pub fn new() -> Self {
        Self {
            output: String::new(),
            _section: PhantomData,
        }
    }

    /// Opens a bucket.
    pub fn bucket(mut self, bucket: &Bucket) -> Writer<InBucket> {
        self.output.push('[');
        self.output.push_str(bucket.as_str());
        self.output.push_str("]\n");

        Writer {
            output: self.output,
            _section: PhantomData,
        }
    }

    /// Finishes the document.
    pub fn finish(self) -> String {
        self.output
    }
}

impl Writer<InBucket> {
    /// Writes one entry into the open bucket.
    pub fn entry(mut self, key: &Key, value: &Value) -> Self {
        self.output.push_str(key.as_str());
        self.output.push_str(" = ");
        self.output.push_str(value.as_str());
        self.output.push('\n');

        self
    }

    /// Closes the open bucket.
    pub fn end(self) -> Writer<Root> {
        Writer {
            output: self.output,
            _section: PhantomData,
        }
    }
}

/// Where a [`Writer`] currently is in the document.
pub trait Section {}

/// A writer between buckets.
pub struct Root;

impl Section for Root {}

/// A writer inside a bucket.
pub struct InBucket;

impl Section for InBucket {}

/// The name of a bucket.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
    fn parsing_a_name_takes_a_whole_line() {
        let mut store = Store::new();
        let users = Bucket::parse("users").unwrap();
        let id = Key::parse("42").unwrap();

        store.insert(users.clone(), id.clone(), Value::new("Alice"));

        assert_eq!(store.get(&users, &id).map(Value::as_str), Some("Alice"));
    }
}
