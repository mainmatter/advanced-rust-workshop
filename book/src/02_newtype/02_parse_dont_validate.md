# Parse, don't validate

`Key` is a distinct type now, but it still accepts anything a `String` accepts: the empty string, a
megabyte of user-supplied bytes, a newline, a null byte. Somewhere downstream, something will care.

The usual answer is a validation function:

```rust
fn is_valid_key(raw: &str) -> bool { /* ... */ }
```

and a rule that everyone calls it before doing anything interesting. This has a specific failure mode:
`is_valid_key` returns a `bool`, and a `bool` is forgotten the instant it goes out of scope. The
compiler has no memory that you checked. Three functions later, someone checks again, defensively,
because they cannot tell whether anyone already did. Six functions later, nobody checks at all.

## The alternative

Move the check into the only path that can produce the type:

```rust
impl Key {
    pub fn parse(raw: &str) -> Result<Self, NameError> { /* ... */ }
}
```

Now the validation returns _evidence_ rather than a verdict. A `Key` value is proof that the check ran
and passed, and that proof travels with the value, through function calls, into structs, across threads.
Downstream code does not re-check, because there is nothing left to check.

This is the difference between **validating** (asking a question and throwing away the answer) and
**parsing** (turning a weakly typed input into a strongly typed output, once, at the edge).

The shape generalises far beyond newtypes:

| Validating                     | Parsing                                    |
| ------------------------------ | ------------------------------------------ |
| `fn is_valid(&str) -> bool`    | `fn parse(&str) -> Result<Key, NameError>` |
| `fn check(&Config) -> bool`    | `fn load(Raw) -> Result<Config, Error>`    |
| `assert!(!v.is_empty())` first | take a `NonEmpty<T>`                       |

## Where the boundary goes

Parse **once, at the edge**: where bytes arrive from a socket, a config file, a CLI argument, a
database row. Everything inside the edge speaks in domain types and never sees a raw `&str` again.

Put differently: the raw type should have the shortest possible lifetime in your program. `&str` comes
in, `Key` comes out, and the `&str` is gone.

## Designing the error

`parse` returns a `Result`, so you need an error type, and you may as well make it a good one:

```rust
pub enum NameError {
    Empty,
    TooLong { length: usize },
    InvalidCharacter { character: char, index: usize },
}
```

Three things worth copying here:

- **One variant per way of failing.** A single `NameError::Invalid(String)` would compile, but callers
  could not distinguish "too long" from "contains a slash" without parsing your prose. Variants are
  matchable, strings are not.
- **Carry the context the caller needs to act.** `TooLong` without a length forces the reader to go
  find the limit. `InvalidCharacter` without an index forces them to hunt for the offending byte.
- **Do not carry the input.** The caller has it: they passed it to you. Cloning it into the error is a
  needless allocation on the error path, and a way to leak user data into logs.

An error is an API, and it is the part of the API your users meet on their worst day.

## Costs, honestly

The type only helps if it is expensive to bypass and cheap to use. That means `parse` should be the
_only_ way in, which is the next section, and it means the type will eventually need `Display`,
`Debug`, `PartialEq`, `Hash` and friends before anyone can comfortably put it in a `HashMap` or an
error message, which is the next chapter.
