# States that move

`ReadOnly` and `ReadWrite` never change: a transaction is born in one and dies in it. The other half
of the pattern is a value that walks through states, where each step changes the type.

The export format has a rule that no signature has expressed so far:

```text
[users]
42 = Alice
43 = Bob
```

Entries belong to a bucket. Writing an entry before opening one is nonsense, and so is finishing the
document while a bucket is still open. Both are easy mistakes and both are, so far, only caught by
reading the output.

## Consuming self is the transition

```rust
impl Writer<Root> {
    pub fn bucket(self, bucket: &Bucket) -> Writer<InBucket>;
    pub fn finish(self) -> String;
}

impl Writer<InBucket> {
    pub fn entry(self, key: &Key, value: &Value) -> Self;
    pub fn end(self) -> Writer<Root>;
}
```

Every method takes `self` and returns the next state. That is the single-use value from chapter 5,
applied once per step: after `bucket()` the `Writer<Root>` is **gone**, moved into a `Writer<InBucket>`,
so there is nothing left to call `finish` on.

The result is a state machine with no runtime representation whatsoever. No enum, no discriminant, no
`match`. The transitions happen at compile time and the generated code is a `String` and some pushes.

Reading the chain out loud is the fastest way to see what has been bought:

```rust
Writer::new()
    .bucket(&users)                        // Root      -> InBucket
    .entry(&alice, &Value::new("Alice"))   // InBucket  -> InBucket
    .end()                                 // InBucket  -> Root
    .finish()                              // Root      -> String
```

Any other order is a compile error, and the compiler names the state you were in when you got it
wrong.

## Where you have already met this

- **`serde`'s serializer.** `serialize_struct` returns a `SerializeStruct`, whose `end()` returns to
  the outer serializer. The nesting rules of the data format are enforced by types, which is how
  `serde` can be format-agnostic and still not let you emit a malformed document.
- **Builders that require fields.** `Builder<Missing, Missing>` gaining parameters as setters are
  called, with `build()` only implemented on `Builder<Set, Set>`. This is how a builder gets
  compile-time required fields instead of an `Option` and a runtime check.
- **`std::process::Command`** is the counter-example worth noting: it does not use typestate, because
  every ordering is legal. Typestate would be pure cost there.

## The costs, again

The same warnings as the previous section, plus one specific to transitions:

**Loops need care.** A `for` loop that calls a transitioning method has to thread the value through:

```rust
let mut open = writer.bucket(bucket);

for (key, value) in entries {
    open = open.entry(key, value);        // reassign, because entry consumed it
}

writer = open.end();
```

That reassignment is the price of the guarantee, and it is the point where people ask whether it was
worth it. For a serializer, where a malformed document is a bug that reaches a customer, it usually
is. For a fluent builder where every order is fine, it is not.

**Conditional transitions are painful.** `if condition { w.bucket(b) } else { w }` does not compile,
because the two branches have different types. When you need that, you are back to an enum and a
runtime check, and that is the honest signal that the state does not belong in the type.
