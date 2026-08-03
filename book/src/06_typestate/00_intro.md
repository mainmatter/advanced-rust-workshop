# Typestate

A new requirement: reporting code should be able to open a transaction that is guaranteed not to
write. Here is the implementation almost every codebase ships:

```rust
pub struct Transaction<'store> {
    // ...
    read_only: bool,
}

pub fn insert(&mut self, bucket: Bucket, key: Key, value: Value) {
    assert!(!self.read_only, "cannot write through a read-only transaction");
    // ...
}
```

It is correct. Every rule is enforced. Count what it costs:

- a `bool` in every transaction, including the ones that will never be read-only;
- a branch on every insert, forever;
- a panic that reaches production if any code path was not covered by a test;
- a `# Panics` section that a caller has to read, believe and remember.

We have spent the whole day moving errors from runtime to compile time. This is a regression, and it
is worth noticing how natural it felt to write.

## The move

**Typestate** means putting the state of a value into its type, so that the operations which are
illegal in that state do not exist.

```rust
pub struct Transaction<'store, S> { /* ... */ }

pub struct ReadOnly;
pub struct ReadWrite;

impl<S> Transaction<'_, S> { pub fn get(&self, ..) -> Option<&Value>; }   // both states

impl Transaction<'_, ReadWrite> { pub fn insert(&mut self, ..); }         // one state
```

`Transaction<'_, ReadOnly>` has no `insert`. Not a private one, not one that panics: there is no
method to call, so the mistake is a compile error at the call site, with a message pointing at the
line that made it.

The `bool` is gone, the branch is gone, and `ReadOnly` and `ReadWrite` are zero-sized, so the struct
does not grow. This is the same trick as the newtype from chapter 2, applied to the state of a value
rather than to its meaning.

## The two shapes

Typestate comes in two flavours, and this chapter does one of each.

**States that never change.** A transaction is born read-only or read-write and stays that way. The
type parameter is a permanent label, and the value never transitions. This is capability narrowing:
the type says what the holder is allowed to do.

**States that change.** A document writer is at the top level, then inside a bucket, then back at the
top level. Each step consumes the value and returns a different type, so the previous state is gone
and cannot be used again. This is a state machine, checked by the compiler, with no runtime
representation at all.

The second shape is where typestate earns its reputation, and it depends entirely on something you
already have: a method taking `self` leaves the caller with nothing.
