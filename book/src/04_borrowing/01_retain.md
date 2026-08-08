# Aliasing XOR mutability

One rule underpins everything in the previous section:

> At any moment, a value may have **either** any number of shared references **or** exactly one
> mutable reference. Never both.

It is often stated as "aliasing xor mutability", and it is the reason the guarantees in the last
section are free. If nobody else can be holding a reference to the store while a transaction has
`&mut`, then isolation is not a property you have to maintain, it is a property the compiler already
proved.

The same rule is also the one that stops you doing this:

```rust
for (bucket, values) in &self.buckets {
    for (key, value) in values {
        if !predicate(bucket, key, value) {
            self.remove(bucket, key);        // error[E0502]
        }
    }
}
```

## Why this is not the compiler being difficult

Removing an entry can make a `HashMap` reallocate its table, which moves every entry. The iterator is
holding a pointer into the old table. In C++ this is undefined behaviour with a name, iterator
invalidation, and it is a reliable source of exploitable bugs. In Java and Python it is a runtime
exception, checked with a modification counter on every step. In Rust it is a compile error and costs
nothing at runtime.

The point worth taking away is that this is not a special case about collections. It is one rule,
applied uniformly, and it is the same rule that gave `minidb` transaction isolation.

## The three ways through

**Two passes.** Collect the decisions first, then act on them. Always works, costs an allocation, and
is the answer when the logic is complicated:

```rust
let doomed = self.buckets.iter()
    .flat_map(|(bucket, values)| values.keys().map(move |key| (bucket.clone(), key.clone())))
    .filter(|(bucket, key)| !predicate(bucket, key, /* ... */))
    .collect::<Vec<_>>();

for (bucket, key) in doomed {
    self.remove(&bucket, &key);
}
```

**The method written from the inside.** `retain` can do in one pass what you cannot do from outside,
because inside the implementation the borrow is not a problem:

```rust
self.buckets.retain(|bucket, values| {
    values.retain(|key, value| predicate(bucket, key, value));
    !values.is_empty()
});
```

Worth internalising as a habit: when the borrow checker rejects a loop that mutates, check whether the
collection already has a method for exactly that shape. `retain`, `retain_mut`, `drain`,
`extract_if`, `entry` and `iter_mut` between them cover most of it.

**Indices instead of references.** An index is not a borrow, so the borrow ends between iterations:

```rust
for i in 0..items.len() {
    if !keep(&items[i]) {
        items.remove(i);      // careful: indices shift
    }
}
```

This is the escape hatch that scales to graphs, where nodes refer to each other by `usize` into an
arena rather than by reference. It also gives up everything the borrow checker was doing for you: an
index into the wrong collection is a logic bug the compiler cannot see.

## When you genuinely need two mutable borrows

Sometimes the requirement is real: two mutable references into the same collection, at once, to
different elements. The compiler cannot prove the elements are distinct, so it refuses.

The standard library solves this by providing the operations from the inside, where `unsafe` can be
used once and reviewed carefully:

```rust
let (left, right) = slice.split_at_mut(mid);
let [a, b] = map.get_disjoint_mut(["x", "y"]);
```

`RefCell` is the other answer: move the check to runtime, and accept a panic if you get it wrong.
That is a real trade rather than a defeat, but reach for it after the two above.

The meta-lesson: the borrow checker is deliberately conservative, so the seams where a safe API is
built on `unsafe` are exactly the places where a rule that is true cannot be proved. `split_at_mut`
exists because that proof is impossible in general and trivial in that one case.
