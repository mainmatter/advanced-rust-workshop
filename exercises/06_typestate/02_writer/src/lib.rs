//! # Exercise
//!
//! `ReadOnly` and `ReadWrite` are states that never change: a transaction is born in one of them and
//! dies in it. This exercise is the other half of the pattern, where the state moves, and the type
//! moves with it.
//!
//! `Store::export` is already written, and it does not compile, because it uses a `Writer` that does
//! not exist yet. `Writer` renders the store as text:
//!
//! ```text
//! [orders]
//! 1 = a book
//! [users]
//! 42 = Alice
//! ```
//!
//! and the format has a rule: entries belong to a bucket. Writing an entry before opening one is
//! nonsense, and so is finishing while a bucket is still open. Encode that in the type.
//!
//! ```text
//! Writer<Root>::new() -> Writer<Root>
//! Writer<Root>::bucket(self, &Bucket) -> Writer<InBucket>     writes "[name]\n"
//! Writer<Root>::finish(self) -> String
//!
//! Writer<InBucket>::entry(self, &Key, &Value) -> Self         writes "key = value\n"
//! Writer<InBucket>::end(self) -> Writer<Root>
//! ```
//!
//! Every method takes `self` and returns the next state, so the old state is gone: this is the
//! single-use value from the last chapter, used once per step to walk a state machine.
//!
//! You will also need `PartialOrd` and `Ord` on `Bucket` and `Key`, because `export` sorts so that
//! the output does not depend on `HashMap` iteration order. One word each.
//!
//! Two things must not compile. An entry outside a bucket:
//!
//! ```compile_fail,E0599
//! use typestate_writer::{Key, Value, Writer};
//!
//! let key = Key::parse("42").unwrap();
//!
//! Writer::new().entry(&key, &Value::new("Alice"));
//! ```
//!
//! and finishing with a bucket still open:
//!
//! ```compile_fail,E0599
//! use typestate_writer::{Bucket, Writer};
//!
//! let users = Bucket::parse("users").unwrap();
//!
//! Writer::new().bucket(&users).finish();
//! ```

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

    /// Lists the buckets that hold at least one value.
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
        todo!()
    }

    /// Opens a bucket.
    pub fn bucket(self, bucket: &Bucket) -> Writer<InBucket> {
        todo!()
    }

    /// Finishes the document.
    pub fn finish(self) -> String {
        todo!()
    }
}

impl Writer<InBucket> {
    /// Writes one entry into the open bucket.
    pub fn entry(self, key: &Key, value: &Value) -> Self {
        todo!()
    }

    /// Closes the open bucket.
    pub fn end(self) -> Writer<Root> {
        todo!()
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
    use crate::{Bucket, Key, Store, Value, Writer};

    #[test]
    fn a_writer_walks_the_states() {
        let users = Bucket::parse("users").unwrap();
        let alice = Key::parse("42").unwrap();
        let bob = Key::parse("43").unwrap();

        let text = Writer::new()
            .bucket(&users)
            .entry(&alice, &Value::new("Alice"))
            .entry(&bob, &Value::new("Bob"))
            .end()
            .finish();

        assert_eq!(text, "[users]\n42 = Alice\n43 = Bob\n");
    }

    #[test]
    fn an_empty_document_is_empty() {
        assert_eq!(Writer::new().finish(), "");
    }

    #[test]
    fn an_empty_bucket_still_gets_a_heading() {
        let users = Bucket::parse("users").unwrap();

        assert_eq!(Writer::new().bucket(&users).end().finish(), "[users]\n");
    }

    #[test]
    fn exporting_sorts_buckets_and_keys() {
        let mut store = Store::new();
        let users = Bucket::parse("users").unwrap();
        let orders = Bucket::parse("orders").unwrap();
        store.insert(users.clone(), Key::parse("43").unwrap(), Value::new("Bob"));
        store.insert(
            users.clone(),
            Key::parse("42").unwrap(),
            Value::new("Alice"),
        );
        store.insert(
            orders.clone(),
            Key::parse("1").unwrap(),
            Value::new("a book"),
        );

        assert_eq!(
            store.export(),
            "[orders]\n1 = a book\n[users]\n42 = Alice\n43 = Bob\n"
        );
    }

    #[test]
    fn exporting_an_empty_store_is_empty() {
        assert_eq!(Store::new().export(), "");
    }
}
