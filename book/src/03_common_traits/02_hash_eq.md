# Eq, Hash and conversions

## The contract

`Hash` and `Eq` are not independent traits. They come with a promise:

> If `a == b`, then `hash(a) == hash(b)`.

`HashMap` relies on it completely. A lookup hashes the key to find a bucket, then compares for
equality inside that bucket. If two equal values hash differently, the second one lands in a different
bucket and the map simply does not find it. There is no panic and no warning: entries go missing, and
the bug reproduces on one machine in ten because it depends on the hasher's random seed.

The classic way to break it is to make equality cleverer than hashing:

```rust
#[derive(Hash)]                                   // hashes the exact bytes
pub struct Key(String);

impl PartialEq for Key {                          // compares case-insensitively
    fn eq(&self, other: &Self) -> bool {
        self.0.eq_ignore_ascii_case(&other.0)
    }
}
```

`Key("A") == Key("a")` is now true, and their hashes differ. Every `HashMap<Key, _>` in the program is
quietly broken.

The rule that follows is blunt and worth following: **derive `Hash` and `PartialEq` together, or write
both by hand.** Never one of each. If you want case-insensitive keys, normalise in `parse` so that the
stored bytes are already canonical, and let the derives stay honest.

`Eq` on top of `PartialEq` is a marker: it promises that equality is reflexive, so `a == a` always
holds. Floats do not qualify, which is why `f64` has `PartialEq` and not `Eq`, and why a struct
containing one cannot be a `HashMap` key.

## Clone and Copy

`Clone` is an explicit, possibly expensive duplicate. `Copy` says the type is duplicated implicitly on
every assignment, which is only appropriate for small, plain data that has no invariant about
uniqueness.

`Key` holds a `String`, so `Copy` is not even an option. But the interesting question is the one you
face when it _is_ possible: adding `Copy` to a public type is a permanent commitment, because removing
it later breaks every caller who relied on a value still being usable after they passed it somewhere.
`Clone` can be added and removed with far less drama.

## From, TryFrom and parse

You already wrote the conversion in chapter 2. `TryFrom` is how you make it discoverable:

```rust
impl TryFrom<&str> for Key {
    type Error = NameError;

    fn try_from(raw: &str) -> Result<Self, Self::Error> {
        Self::parse(raw)
    }
}
```

Two things follow for free, which is the whole reason to bother:

- **`TryInto` comes with it.** A blanket impl in `std` gives you `raw.try_into()` on the calling side,
  and it is the form that works when the target type is inferred rather than named.
- **Generic code can call it.** A function taking `T: TryFrom<&str>` works with your type without
  knowing it exists. This is how `clap`, `serde` and friends accept domain types.

Keep the inherent `parse` as well. It is discoverable in the rustdoc method list, it does not need the
trait in scope, and it gives a better error message when someone gets the types wrong.

`From` is the infallible sibling, and there is one rule about it: **`From` must never panic**. If the
conversion can fail, it is `TryFrom`. An `impl From<&str> for Key` that unwraps internally is a
landmine, because callers reasonably assume `.into()` cannot blow up.

The other direction is free and worth adding: `impl From<Key> for String` gives you `.into()` where
`into_inner()` reads awkwardly, and costs nothing since you have the method already.

## The rest of the list

For completeness, and none of it deserves more than a line here:

- **`Display`**: covered in the previous section. No derive, and that is deliberate.
- **`PartialOrd` and `Ord`**: derive them if the field order happens to be the order you want, which
  for a single-field newtype it always is. Same contract trap as `Hash`: if you hand-write `Ord`, make
  it consistent with `Eq`.
- **`Default`**: only if "empty" is a meaningful value. For a type with an invariant it usually is not,
  and `#[derive(Default)]` on a validated newtype is another door around `parse`.
- **`Serialize` and `Deserialize`**: covered in chapter 2. The derive on `Deserialize` bypasses your
  constructor, so use `#[serde(try_from = "String")]`.
