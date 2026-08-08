//! # Exercise
//!
//! Nothing to write here. Read the doctests, run `wr`, and move on.
//!
//! Rust has two kinds of reference, and what separates them is aliasing, not writing:
//!
//! - `&Store` is a **shared reference**. Any number of them may exist at once.
//! - `&mut Store` is an **exclusive reference**. While one exists, no other reference to that value
//!   does.
//!
//! Calling them "immutable" and "mutable" is the usual mistake. `&Cell<T>`, `&Mutex<T>` and
//! `&AtomicUsize` all let you change the value through a shared reference. What `&mut` promises is
//! that no other path to the value exists, and being allowed to change things is what that promise
//! buys.
//!
//! The rule is one line long: **any number of shared references, or exactly one exclusive
//! reference, never both**. Aliasing XOR mutation. Hold a value borrowed out of the store, then
//! change the store, and you are asking for both:
//!
//! ```compile_fail,E0502
//! use borrowing_intro::{Bucket, Key, Store, Value};
//!
//! let mut store = Store::new();
//! let users = Bucket::parse("users").unwrap();
//! let id = Key::parse("42").unwrap();
//! store.insert(&users, &id, Value::new("Alice"));
//!
//! let alice = store.get(&users, &id);
//!
//! store.insert(&users, &id, Value::new("Bob"));
//!
//! println!("{alice:?}");
//! ```
//!
//! Two exclusive references are the same rule from the other side:
//!
//! ```compile_fail,E0499
//! use borrowing_intro::Store;
//!
//! let mut store = Store::new();
//!
//! let first = &mut store;
//! let second = &mut store;
//!
//! first.buckets().count();
//! second.buckets().count();
//! ```
//!
//! Every reference is valid over a **region** of the code, and a **lifetime** is the name of that
//! region. The region ends at the reference's last use, not at the closing brace, which is why
//! moving one line makes the first example compile:
//!
//! ```
//! use borrowing_intro::{Bucket, Key, Store, Value};
//!
//! let mut store = Store::new();
//! let users = Bucket::parse("users").unwrap();
//! let id = Key::parse("42").unwrap();
//! store.insert(&users, &id, Value::new("Alice"));
//!
//! let alice = store.get(&users, &id).map(Value::as_str);
//! assert_eq!(alice, Some("Alice"));
//!
//! store.insert(&users, &id, Value::new("Bob"));
//! ```
//!
//! That is **non-lexical lifetimes**, and it is younger than the language: before Rust 2018 a
//! borrow really did run to the end of its block, and this version had to be written with an extra
//! scope around the read.
//!
//! You seldom write a lifetime down, because most of them are inferred. `Store::get` is declared
//!
//! ```text
//! pub fn get(&self, bucket: &Bucket, key: &Key) -> Option<&Value>
//! ```
//!
//! and means
//!
//! ```text
//! pub fn get<'s, 'b, 'k>(&'s self, bucket: &'b Bucket, key: &'k Key) -> Option<&'s Value>
//! ```
//!
//! Three rules do that, and they are the whole of **lifetime elision** for functions: every elided
//! input lifetime becomes its own parameter; if there is exactly one input lifetime, every elided
//! output gets it; and if one input is `&self`, every elided output gets `self`'s lifetime instead.
//! The `Formatter<'_>` you wrote in chapter 3 is the third spelling, the **anonymous lifetime**:
//! there is a borrow in that type and it is not worth naming.
//!
//! None of those rules apply to a struct that holds a reference, where you have to write the
//! lifetime yourself. That is chapter 5.
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
    fn any_number_of_shared_references_at_once() {
        let mut store = Store::new();
        let users = Bucket::parse("users").unwrap();
        let id = Key::parse("42").unwrap();
        store.insert(&users, &id, Value::new("Alice"));

        let first = store.get(&users, &id);
        let second = store.get(&users, &id);
        let buckets = store.buckets().count();

        assert_eq!(first.map(Value::as_str), Some("Alice"));
        assert_eq!(second.map(Value::as_str), Some("Alice"));
        assert_eq!(buckets, 1);
    }
}
