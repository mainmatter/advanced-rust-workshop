# Extension traits

Every ergonomic improvement so far has been a method on a type we own. Chapter 2 gave us `Key` and
`Bucket`, and every call site since has spelled the constructor out:

```rust
let users = Bucket::parse("users")?;
let id = Key::parse("42")?;
```

What we want is the method on the string:

```rust
let users = "users".to_bucket()?;
let id = "42".to_key()?;
```

This is the first wish in the course that cannot be granted by adding a method, because `str` belongs
to the standard library. Two separate rules stand in the way, and only the second one is the orphan
rule.

## The rule you hit first

```rust
impl str {
    fn to_key(&self) -> Result<Key, NameError> { .. }   // error[E0390]
}
```

Inherent impls may only be written in the crate that defines the type: `E0116` in general, and
`E0390` for a primitive like `str`. No coherence argument is involved here. The rule is only about
where a type's own methods live.

## The orphan rule

The rule that shapes the rest of the chapter is the other one. You may implement a trait for a type
only if **you own the trait or you own the type**. Both foreign is `E0117`:

```rust
impl Display for Option<String> { .. }   // error[E0117]
```

The reason is coherence. If two crates could both `impl Display for Vec<u8>`, then a program depending
on both would have two implementations for the same call and no principled way to choose. Rust's
answer is to make the situation impossible rather than to define a tie-break, which is also why
adding an impl in a library is a semver-visible act.

## The way through

You own the trait if you define it. So define one:

```rust
pub trait StrExt {
    fn to_key(&self) -> Result<Key, NameError>;
}

impl StrExt for str {
    fn to_key(&self) -> Result<Key, NameError> {
        Key::parse(self)
    }
}
```

Now `"users/42".to_key()?` works, on a type you do not own, without breaking coherence: your crate
owns `StrExt`, and any other crate's competing extension trait is a different trait.

This is an **extension trait**, and the convention is to name it after what it extends with an `Ext`
suffix: `StrExt`, `IteratorExt`, `ResultExt`.

## The catch that is also the point

An extension trait's methods exist only where the trait is in scope:

```rust
use minidb::StrExt;      // without this line, `to_key` does not exist
```

That is not a wart, it is the mechanism. Your extension methods cannot collide with anybody else's
unless a caller deliberately imports both, which is why `use itertools::Itertools;` is a line you
write rather than something that happens to you.
