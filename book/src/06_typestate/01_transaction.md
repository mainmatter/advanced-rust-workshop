# States as capabilities

The mechanics are small. Two pieces.

**Marker types.** Zero-sized structs whose only job is to be distinct:

```rust
pub struct ReadOnly;
pub struct ReadWrite;
```

**`PhantomData`** to use a type parameter you store no value of:

```rust
pub struct Transaction<'store, A> {
    store: &'store mut Store,
    undo: Vec<Undo>,
    finished: bool,
    _access: PhantomData<A>,
}
```

Rust requires every type parameter to appear in the body, and `PhantomData<A>` is how you satisfy
that without storing anything. It is zero-sized: `size_of::<Transaction<ReadWrite>>()` and
`size_of::<Transaction<ReadOnly>>()` are the same, and both are the size of the fields you actually
have.

Then split the methods:

```rust
impl<A> Transaction<'_, A> {
    pub fn get(&self, ..) -> Option<&Value> { .. }      // every mode
}

impl Transaction<'_, ReadWrite> {
    pub fn insert(&mut self, ..) { .. }                 // one mode
    pub fn commit(self) { .. }
}
```

## Two things that will catch you

**`Drop` must match the struct exactly.** A `Drop` impl has to repeat its struct's bounds, and it
cannot add one the struct does not have. Our struct has none, so neither may the destructor:

```rust
impl<A> Drop for Transaction<'_, A> { .. }      // add `where A: ..` here and you get E0367
```

Chapter 8 adds a bound to the struct, and all three sites, struct, shared impl and `Drop`, have to
gain it together.

This is also why you cannot write a `Drop` impl for only one mode. If the two modes need different
destructor behaviour, the difference has to live in a field, which is exactly what `finished` does
here: a read-only transaction is born finished, so the drop bomb never arms for it.

**Nothing yet says what `A` may be.** `Transaction<'_, u32>` is a nameable type, and it has no methods
and no way to be constructed, so it is a curiosity rather than a hole. Writing the set down takes a
trait, and deciding who may add to it takes sealing. Both are chapter 8.

## Capabilities and tokens

`Transaction<'_, ReadWrite>` is doing something more general than tracking a state: **holding it is
proof that you are allowed to write.** A function taking one does not need to check anything, because
possession is the check.

That idea has its own name, the **permission token** or capability, and once you see it you find it
everywhere:

- **`MutexGuard<T>`** is a token that proves the lock is held, and it carries the data so that you
  cannot reach the data without it. Token plus payload is the most useful form.
- **Embedded HALs** hand out a `Peripherals` struct exactly once per program, via `take()`. Owning the
  `Pin<Output>` is proof that nobody else has configured that pin.
- **Zero-sized proof tokens** are the pure form:

  ```rust
  pub struct Authenticated(());        // private field: only this module can build one

  pub fn authenticate(creds: &Credentials) -> Option<Authenticated>;
  pub fn delete_everything(_: &Authenticated);
  ```

  `delete_everything` cannot be called without an `Authenticated`, and an `Authenticated` cannot be
  built without going through `authenticate`. The private `()` field is what closes the door, exactly
  as it did for `Key` in chapter 2.

The token pattern and typestate are the same idea seen from two angles: a type that exists only to
carry a fact the compiler can check.

## When not to do this

Typestate is not free, and the costs land on your users:

- **Error messages get worse.** "no method named `insert` found for struct
  `Transaction<'_, ReadOnly>`" is good. The equivalent in a five-parameter generic builder is not.
- **The type parameter is contagious.** Every function that takes your type either fixes the state or
  becomes generic over it, and that spreads.
- **`dyn` becomes awkward.** `Box<dyn Transaction>` does not exist any more, because there is no one
  type. If callers need to store your value in a collection alongside other states, typestate fights
  them.
- **Two states are sometimes just two types.** If `ReadOnly` and `ReadWrite` shared no methods at all,
  two separate structs would be simpler and clearer than a type parameter.

The rule of thumb: reach for typestate when the states share most of their behaviour, when the illegal
operations are genuinely illegal rather than merely unusual, and when the value is used directly rather
than through a trait object.
