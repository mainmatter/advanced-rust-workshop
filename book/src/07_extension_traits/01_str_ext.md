# Writing one, and not writing one

## Implement it for the unsized type

```rust
impl StrExt for str { .. }       // yes
impl StrExt for &str { .. }      // no
impl StrExt for String { .. }    // no
```

Implementing for `str` gets you all three. A `&str` finds the method by auto-deref, and so does a
`String`, because `String: Deref<Target = str>`. Implementing for `&str` instead gets you exactly one
of the three and a confusing error for the other two.

The same rule applies elsewhere: implement for `[T]`, not `&[T]`; for `Path`, not `&Path`.

## Now do not write it

Before defining an extension trait, check whether the standard library already has the trait you are
about to reinvent. For this exact case it does:

```rust
impl FromStr for Key {
    type Err = NameError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        Self::parse(raw)
    }
}
```

That gives callers `"users/42".parse::<Key>()`, and `.parse()` with no turbofish wherever the target
type is inferred. It requires no import, because `FromStr` is in the prelude, and it is the spelling
every Rust programmer already knows.

It also unlocks things you did not write: `clap` uses `FromStr` for argument parsing, `serde` can be
pointed at it, and a generic function taking `T: FromStr` accepts your type without knowing it exists.

**Prefer the standard trait.** The list worth checking before inventing anything:

| You want            | The trait that already exists |
| ------------------- | ----------------------------- |
| parse from a string | `FromStr`                     |
| convert, fallibly   | `TryFrom`                     |
| convert, infallibly | `From`                        |
| render for humans   | `Display`                     |
| borrow as something | `AsRef`, `Borrow`             |
| iterate             | `IntoIterator`                |
| a default value     | `Default`                     |

An extension trait is what you reach for when nothing in that list fits, or when the method genuinely
has no home other than "convenience on somebody else's type".

## So when is it right?

- **Convenience layers over a foreign API.** `ResultExt::context()` in `anyhow` adds an operation to
  every `Result` in your program that the standard library was never going to add.
- **Methods on a trait you do not own.** `Itertools` and `FuturesExt` are the canonical examples, and
  they are the subject of the next section.
- **Keeping a public type small.** An extension trait in a companion crate lets people opt into an
  ergonomic layer without it appearing in the core type's documentation.

And when it is not:

- **On your own type.** If you own it, add an inherent method. Inherent methods need no import, show
  up first in the rustdoc, and take priority in method resolution.
- **To reach around a missing API.** If the upstream type is missing something fundamental, an
  extension trait in your crate is a private fix for a public problem. Consider sending a patch.
