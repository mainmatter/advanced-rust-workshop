# Doc comments

A doc comment is written for someone who can see your signature and cannot see your body. That single
constraint decides almost everything about what belongs in one.

## Anatomy

````rust
/// Inserts a value, returning the value it replaced, if any.
///
/// Values are stored per bucket, so the same key in two buckets is two values.
///
/// # Errors
///
/// Returns [`NameError`] if the key is empty, over 64 bytes, or contains
/// characters outside `[A-Za-z0-9._/-]`.
///
/// # Examples
///
/// ```
/// let mut store = Store::new();
/// assert_eq!(store.insert("users", "42", "Alice"), None);
/// ```
pub fn insert(&mut self, bucket: &str, key: &str, value: &str) -> Option<String>
````

- **The summary line** is one sentence, in the third person, ending with a full stop. It shows up in
  the type's method list, so it has to stand alone: `rustdoc` will show it next to forty others.
- **The body** is for what the caller cannot infer: the surprising bit, the invariant, the relationship
  to the neighbouring method.
- **`# Errors`** lists the conditions, not just the type. "Returns `Err`" tells a caller nothing they
  could not read off the signature.
- **`# Panics`** is the one section people skip and the one that costs them. If your function can
  panic, that is part of its contract, and a caller who does not know cannot defend against it.
- **`# Examples`** comes last, and is the only part of the whole comment that cannot silently rot.

## What and why, not how

The comment that hurts is the one that restates the body:

```rust
/// Loops over the buckets and sums their lengths.
pub fn len(&self) -> usize
```

The caller does not care, and the sentence becomes false the moment someone caches the count. Write
instead what it is for and what it costs:

```rust
/// Returns the number of values across every bucket.
///
/// This walks every bucket, so it is O(number of buckets), not O(1).
pub fn len(&self) -> usize
```

## Examples are tests

`cargo test` compiles and runs every example in every doc comment. This is worth more than it sounds:

- an example that no longer compiles is a **failing test**, so your documentation cannot drift out of
  sync with your API without someone noticing;
- an example is the only part of the docs that is proven to be true;
- writing one is the fastest way to discover that your own API is annoying to call.

The corollary is that a rotted example is worse than no example, because it is a test that nobody ran.
If a doc example does not survive a rename, either the rename or the example was wrong.

Hidden lines starting with `#` let you keep an example short without making it a lie:

````rust
/// ```
/// # use minidb::Store;
/// let mut store = Store::new();
/// store.insert("users", "42", "Alice");
/// ```
````

The `use` runs, so the example is real, but it does not clutter the rendered page.

## Link to your own types

Square brackets make intra-doc links: `[`NameError`]` resolves to the type, and `rustdoc` will warn
you if it stops resolving. `#![deny(rustdoc::broken_intra_doc_links)]` turns that warning into a build
failure, which is how you keep the cross-references honest.

## How much is enough

Not every method deserves four sections. A reasonable floor:

- every public item has a summary line, and `#![deny(missing_docs)]` is how you make that true rather
  than aspirational;
- anything returning `Result` has `# Errors`;
- anything that can panic has `# Panics`;
- anything whose use is non-obvious has an example.

Everything beyond that is a judgement call, and the failure mode is not "too little documentation" but
prose that repeats the signature in longer words.
