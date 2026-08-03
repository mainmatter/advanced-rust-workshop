# States as capabilities

The mechanics are small. Three pieces.

**Marker types.** Zero-sized structs whose only job is to be distinct:

```rust
pub struct ReadOnly;
pub struct ReadWrite;
```

**A trait to group them**, so `Transaction<u32>` cannot be written:

```rust
pub trait State {}

impl State for ReadOnly {}
impl State for ReadWrite {}
```

**`PhantomData`** to use a type parameter you store no value of:

```rust
pub struct Transaction<'store, S>
where
    S: State,
{
    store: &'store mut Store,
    undo: Vec<Undo>,
    finished: bool,
    _state: PhantomData<S>,
}
```

Rust requires every type parameter to appear in the body, and `PhantomData<S>` is how you satisfy
that without storing anything. It is zero-sized: `size_of::<Transaction<ReadWrite>>()` and
`size_of::<Transaction<ReadOnly>>()` are the same, and both are the size of the fields you actually
have.

Then split the methods:

```rust
impl<S> Transaction<'_, S>
where
    S: State,
{
    pub fn get(&self, ..) -> Option<&Value> { .. }      // every state
}

impl Transaction<'_, ReadWrite> {
    pub fn insert(&mut self, ..) { .. }                 // one state
    pub fn commit(self) { .. }
}
```

## Two things that will catch you

**`Drop` must match the struct exactly.** If the struct says `where S: State`, so must the `Drop`
impl, and it cannot add a bound the struct does not have:

```rust
impl<S> Drop for Transaction<'_, S>
where
    S: State,          // must be identical to the struct's bounds
{ .. }
```

This is why you cannot write a `Drop` impl for only one state. If different states need different
destructor behaviour, the difference has to live in a field, which is exactly what `finished` does
here: a read-only transaction is born finished, so the drop bomb never arms for it.

**The trait should be sealed.** As written, a downstream crate can `impl State for MyType` and get a
`Transaction<MyType>` that satisfies neither impl block. Sealing prevents that, and it is chapter 8's
material, so we leave the trait open for now with the note that a real library would not.

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
