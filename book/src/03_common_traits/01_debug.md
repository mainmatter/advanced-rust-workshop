# Debug and Display

Two traits, two audiences, and the distinction is worth getting right because it decides which one you
reach for at three in the morning.

**`Debug`** is for programmers. It is `{:?}`, it shows structure, it is allowed to be ugly, and it
should round-trip your mental model of the value. Derive it on virtually everything.

**`Display`** is for the people using your program. It is `{}`, there is no derive, and the absence of
a derive is the point: a human-facing rendering is a decision, not a projection of your field names.

A rule of thumb that survives contact with reality: if you cannot say who reads the output, you want
`Debug`.

```rust
use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub struct Key(String);          // Key("users/42")

impl Display for NameError {     // "key is empty"
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result { /* ... */ }
}
```

## Debug is a security surface

Here is the part people learn the hard way.

`Debug` output does not stay where you put it. It ends up in log lines, in panic messages, in
`unwrap()` failures, in test output pasted into a ticket, in an error report shipped to a third-party
service. Anything reachable by `Debug` from a struct you log is, effectively, logged.

So for a type holding data you do not own, deriving `Debug` is a decision:

```rust
pub struct Value(String);

impl Debug for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Value(<redacted, {} bytes>)", self.0.len())
    }
}
```

The `'_` in `Formatter<'_>` is the anonymous lifetime: there is a borrow inside that type, and naming
it would buy nothing. Chapter 4 says what it is short for.

The length is not an accident. A redaction that shows nothing at all makes debugging genuinely harder,
and people respond by removing it. Showing the length distinguishes an empty value from a truncated one
and from a value that is there but wrong, which covers most of what you actually need, and tells a
reader of your logs nothing they can use.

This is the same reasoning behind [`secrecy`](https://docs.rs/secrecy)'s `Secret<T>`, and behind
`std`'s decision that `OsStr` and `Path` print quoted and escaped rather than raw.

The failure mode to watch for is indirect: a type with a careful `Debug` impl held inside a struct that
derives `Debug` is safe, because the derive calls your impl. A type with a careful impl whose data is
_also_ reachable through some other public accessor that gets logged is not. Redaction protects a
field, not a value.

## Writing the impls

For the common cases you rarely need to touch a `Formatter` directly:

```rust
impl Debug for Config {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Config")
            .field("url", &self.url)
            .field("token", &"<redacted>")
            .finish_non_exhaustive()
    }
}
```

`debug_struct`, `debug_tuple`, `debug_list` and `debug_map` handle the formatting flags for you, so
`{:#?}` still pretty-prints. `finish_non_exhaustive` renders the `..` that tells a reader you left
something out on purpose.

One rule for `Display`: never write `\n` into it. The caller decides about layout, and a `Display` impl
that emits a newline is unusable inside a larger message.
