# Semantic confusion

A **newtype** is a tuple struct with exactly one field:

```rust
pub struct Bucket(pub String);
pub struct Key(pub String);
```

That is the whole pattern. `Bucket` and `Key` hold the same data as before and behave the same at
runtime, but they are now different types, and `Store::get(&self, bucket: &Bucket, key: &Key)` can no
longer be called with its arguments the wrong way round.

The bug from the previous section stops being a bug you find in production and becomes a bug you find
while typing.

## It really is free

A newtype with a single field has the same size, alignment and representation as the field itself.
There is no wrapper object, no indirection, no allocation. After monomorphisation and inlining, code
that passes a `Key` around compiles to exactly the code that passed a `String` around.

You pay in source code, not in cycles: the wrapping, the unwrapping, and the trait impls you now have
to write yourself. That last cost is real, and we will spend the next chapter on it.

## A type alias is not a newtype

This is the tempting shortcut, and it does nothing:

```rust
type Key = String;
type Bucket = String;
```

A type alias introduces a new _name_, not a new _type_. `Key` and `Bucket` are both still `String`,
they are still interchangeable, and the swapped-argument bug still compiles. Aliases are for shortening
`Result<T, std::io::Error>`, not for encoding meaning.

The same goes for the other near miss:

```rust
pub struct Key(pub String);

fn get(bucket: &str, key: &str)  // still takes two `&str`
```

Defining the type is only half the work. The type has to reach the signature.

## When not to do it

Newtypes have a cost at every boundary they cross, so they are not free in a codebase, only on the CPU.
A rough rule:

- **Wrap it** when the value has domain meaning that the underlying type does not capture (`Key`,
  `UserId`, `Celsius`, `Bytes`), when confusing it with its neighbours is plausible, or when it will
  grow an invariant later. Whether it will grow an invariant later is easier to predict than you think.
- **Leave it alone** when the underlying type already says everything (`fn len(&self) -> usize`), or
  when the value is genuinely just data passing through.

The signal to watch for is two or more parameters of the same type sitting next to each other in a
signature. It is not a proof of a problem, but it is where the problems live.
