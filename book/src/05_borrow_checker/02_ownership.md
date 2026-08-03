# Ownership in signatures

Here is `minidb`'s `insert`, as you have been using it all day:

```rust
pub fn insert(&mut self, bucket: &Bucket, key: &Key, value: Value) -> Option<Value> {
    self.buckets
        .entry(bucket.clone())
        .or_default()
        .insert(key.clone(), value)
}
```

It borrows, and then immediately clones, because a map has to own its keys. The signature says "lend
me these"; the body says "mine now". Two allocations per insert that the caller cannot see in the
signature, cannot avoid, and is not told about.

This is the pattern to learn to spot: **a `&T` parameter that is cloned in the body is a hidden
cost**. It looks polite and it is not.

## Say what you need

The rule is boring and worth following: **if the function needs ownership, ask for ownership.**

```rust
pub fn insert(&mut self, bucket: Bucket, key: Key, value: Value) -> Option<Value>
```

Three things improve at once. The cost moves to the call site, where the caller can see the `clone`
and decide whether they needed it. A caller who already has an owned value stops paying for a copy
they did not need. And the signature stops lying.

The standard library is consistent about this, and it is worth reading the asymmetry deliberately:

```rust
impl<K, V> HashMap<K, V> {
    pub fn insert(&mut self, k: K, v: V) -> Option<V>;          // stores it: takes it
    pub fn get<Q>(&self, k: &Q) -> Option<&V>;                  // looks at it: borrows it
    pub fn remove<Q>(&mut self, k: &Q) -> Option<V>;            // looks it up, hands back the value
}
```

Nothing there is an accident, and the shape of each signature tells you what happens to your data.

## The middle ground

Taking ownership pushes `clone()` calls onto callers, which is honest but can get noisy. Two ways to
soften it, both with real costs:

**`impl Into<T>`** lets a caller pass whatever they have, and the conversion happens inside:

```rust
pub fn insert(&mut self, bucket: impl Into<Bucket>, ...)
```

The cost is not zero: it is still a conversion, it just moved. It also makes the signature harder to
read and it does not work for a type with a validating constructor, because `From` must not fail.

**`Cow<'a, T>`** lets one function serve both callers, borrowing when it can and owning when it must.
It is the right answer for a parser or normaliser that usually passes data through unchanged, and it
is overkill almost everywhere else. Reach for it when profiling says so, not before.

## Owned and view types

The larger version of this idea is that many domains want a **pair** of types: one that owns and one
that borrows.

| Owns       | Views            | Notes                                     |
| ---------- | ---------------- | ----------------------------------------- |
| `String`   | `&str`           | the original                              |
| `PathBuf`  | `&Path`          | unsized view, not just a reference        |
| `Vec<T>`   | `&[T]`           |                                           |
| `OsString` | `&OsStr`         |                                           |
| `OwnedFd`  | `BorrowedFd<'_>` | the view carries a lifetime but is `Copy` |

The convention that falls out of it, and it is one of the most reliable rules in Rust API design:

> **Take the view type as a parameter, return the owned type.**

A function taking `&str` can be called by anyone holding a `String`, a `&str`, or a string literal. A
function taking `&String` can only be called by someone who happens to have a `String`, and gains
nothing for the restriction. The same argument applies to `&[T]` over `&Vec<T>`, and to `&Path` over
`&PathBuf`.

`minidb` does not need a separate `KeyRef` type, because `&Key` already does the job: a `Key` is a
thin wrapper and a shared reference to it is a perfectly good view. The pattern earns its keep when
the owned type has structure the view does not need, which is exactly why `Path` exists and
`&PathBuf` is a code smell.

`OwnedFd` and `BorrowedFd` are the version worth studying if you write anything that wraps a resource
with a lifetime, because there the distinction is about who closes the file descriptor, and getting it
wrong is a use-after-close rather than a wasted allocation. We come back to how `BorrowedFd` is built
in the last chapter.
