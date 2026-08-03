# Common traits

The last chapter took a `String` and wrapped it. Wrapping does not only add: it takes away.

A `String` can be printed, compared, sorted, cloned and used as a key in a `HashMap`. `Key(String)` can
do none of those, because a newtype starts life with no traits at all. Three lines that used to work:

```rust
println!("{key:?}");                          // Key does not implement Debug
let same = a == b;                            // no PartialEq
map.insert(key, "Alice");                     // no Hash, no Eq
```

This is the bill for the newtype pattern, and it is the reason people abandon it halfway. It arrives as
a compiler error at the exact moment you are trying to do something else.

## The good news

Most of the bill is paid with one line:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Key(String);
```

So this chapter is short. It is about the four decisions in it that `derive` cannot make for you:

- **`Debug` on a type that holds user data.** Deriving it is a decision about what ends up in your logs.
- **`Hash` and `Eq` together.** They have a contract, and hand-writing one of the two breaks it
  silently.
- **`Clone` versus `Copy`.** One of them changes how your API feels to use.
- **`From` and `TryFrom`.** The generic entry points to the parsing you already wrote.

Everything else on the list, `Display`, `PartialOrd`, `Ord`, is either a `derive` or a five-line impl,
and we will not spend the morning on it.
