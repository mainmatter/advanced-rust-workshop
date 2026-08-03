# Extension traits

Every ergonomic improvement so far has been a method on a type we own. That option disappears the
moment the type belongs to somebody else:

```rust
impl str {
    fn to_key(&self) -> Result<Key, NameError> { .. }   // no
}
```

You cannot add an inherent method to `str`, or to `Option`, or to anything you did not define.

## The orphan rule

You may implement a trait for a type only if **you own the trait or you own the type**. Both foreign
is forbidden, and the error is `E0117`.

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
