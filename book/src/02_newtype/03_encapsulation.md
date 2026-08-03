# Is it encapsulated?

`Key::parse` rejects invalid keys. This still compiles:

```rust
let key = Key(String::new());
```

And so does this:

```rust
let mut key = Key::parse("users/42").unwrap();
key.0.push('\n');
```

The invariant lasted exactly as long as it took someone to reach around it. An invariant enforced by a
constructor that anyone can bypass is a convention, and conventions do not survive contact with a
deadline.

## The rule

**An invariant holds only if every path that can construct or mutate the value goes through code that
checks it.** In Rust that means the field is private, and the module boundary does the rest:

```rust
pub struct Key(String);

impl Key {
    pub fn parse(raw: &str) -> Result<Self, NameError> { /* ... */ }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}
```

Note what changed and what did not. Inside the defining module, `self.0` still works, which is why
`Store` needs no rewrite. Outside it, `Key` has exactly one door, and `parse` is standing in it.

## Giving data back

Two accessors, and the difference between them is worth being deliberate about:

- `as_str(&self) -> &str` hands out a read-only view. The caller can look, cannot touch, and the `Key`
  stays valid.
- `into_inner(self) -> String` hands over the data and consumes the `Key`. The caller can now do
  anything at all to that `String`, and it does not matter: it is not a `Key` any more. The only way
  back is `parse`.

The one you should not write is `as_mut(&mut self) -> &mut String`. It hands out unrestricted mutation
of a value that is still a `Key`, which is the same hole as a public field with more ceremony. If
callers need to modify a key, give them an operation that preserves the invariant, or make them go
through `parse` again.

## Do not reach for `Deref`

Sooner or later somebody suggests this:

```rust
impl Deref for Key {
    type Target = String;
    fn deref(&self) -> &String { &self.0 }
}
```

It is seductive: every `String` method appears on `Key` for free, and the wrapping stops feeling like
work. It is also a mistake for a newtype like this one:

- it re-exports the entire `String` API as though it were `Key`'s API, including methods that make no
  sense for a key and methods that will be added to `String` in future releases;
- with `DerefMut`, it hands back exactly the mutation hole you just closed;
- deref coercion is implicit, so `Key` starts silently coercing to `String` in ways that undo the type
  distinction you built the newtype for.

`Deref` is for smart pointers, types whose whole purpose is to stand in for something else: `Box<T>`,
`Rc<T>`, `MutexGuard<T>`. A newtype is the opposite. Its purpose is to _not_ be the thing it wraps.

Implement the handful of methods your callers actually need. It is more typing and a smaller API, and
a smaller API is the product.

## The hole nobody notices

Once your newtype has an invariant, every trait that can construct it from the outside is a new door.
The most common one is `serde`:

```rust
#[derive(Deserialize)]  // reads the raw string straight into the field
pub struct Key(String);
```

`#[derive(Deserialize)]` bypasses `parse` entirely, which means a JSON payload can hand you a `Key`
that `parse` would have rejected. If a type has an invariant, its `Deserialize` impl has to go through
the same door as everyone else, using `#[serde(try_from = "String")]` or a hand-written impl.

The same question is worth asking of any trait you derive on a type with an invariant: can this
construct a value, or mutate one, without passing my check? `Default` frequently can. So can a careless
`From`.
