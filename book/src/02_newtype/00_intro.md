# The newtype pattern

Here is the store from the previous chapter, trimmed to the four operations we will spend the rest of
the day evolving:

```rust
pub struct Store {
    buckets: HashMap<String, HashMap<String, String>>,
}

impl Store {
    pub fn insert(&mut self, bucket: &str, key: &str, value: &str) -> Option<String> { /* ... */ }
    pub fn get(&self, bucket: &str, key: &str) -> Option<&str> { /* ... */ }
    pub fn remove(&mut self, bucket: &str, key: &str) -> Option<String> { /* ... */ }
}
```

There is nothing wrong with this code. It compiles, it is easy to read, it does what it says. It is
also the version of the library that will generate support tickets for the next two years.

## Three strings walk into a function

`insert` takes three `&str` parameters. To the compiler they are interchangeable. To the domain they
are nothing of the sort: the first names a partition, the second names a value inside it, the third
_is_ the value. Get the order wrong and the compiler waves you through:

```rust
let mut store = Store::new();
store.insert("users", "42", "Alice");

// Later, in a different file, written by a different person, at 17:45 on a Friday:
store.get("42", "users")  // => None
```

No panic. No error. Just `None`, which the caller will dutifully interpret as "no such user", because
that is what `None` means everywhere else in this API.

The information needed to catch this exists. It is in the parameter names, in the doc comment, and in
the head of whoever wrote `insert`. The only place it is _not_ is in the type system, which is the one
place the compiler can read.

## What this chapter is about

Three steps, each one a small, unglamorous change:

1. **Give distinct things distinct types.** The compiler cannot help you tell `bucket` from `key` until
   they stop being the same type.
2. **Parse, don't validate.** A type that can only be built from valid input turns "is this key
   legal?" from a question you keep asking into a question you answered once.
3. **Close the back door.** An invariant enforced by a constructor that anyone can bypass is a
   convention, not an invariant.

None of this is clever. That is the point: it is the cheapest correctness you will ever buy in Rust,
and most codebases leave it on the table.
