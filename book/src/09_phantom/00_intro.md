# PhantomData, variance and brands

You have been using `PhantomData` since chapter 6 without asking what it does.

The usual explanation is "it silences the unused type parameter error", which is true and is the least
interesting part. The real definition:

> `PhantomData<T>` makes the compiler treat your struct as though it contained a `T`, for every
> purpose except memory.

Four purposes, specifically.

**Size**: none. `PhantomData<T>` is zero-sized for every `T`, including `T` that are enormous.
`Transaction<'_, ReadOnly>` and `Transaction<'_, ReadWrite>` have identical layouts.

**Auto traits**: `Send`, `Sync` and friends are decided by what a type contains, and `PhantomData`
counts as containing:

```rust
struct Handle {
    id: u32,
    _marker: PhantomData<*const ()>,      // now !Send and !Sync
}
```

That is how a type that is nothing but an integer can be made thread-bound, which matters for handles
that are only valid on the thread that created them.

**Ownership and drop check**: `PhantomData<T>` tells the compiler you own a `T`, so the borrow checker
treats your struct as though it will drop one. This matters for types built on raw pointers, where
the compiler otherwise cannot see that dropping your struct might touch borrowed data.

**Variance**: whether `Foo<'long>` may be used where `Foo<'short>` is expected. This one has no
observable effect until it does, and then it is the entire trick behind the last exercise of the day.

## Where the chapter goes

Two exercises, both of them the same move: **use a marker to claim a relationship the data does not
have.**

1. A handle that owns its data and borrows nothing, made to behave exactly like a borrow. This is how
   `BorrowedFd` works, and it is the promise from chapter 5.
2. A lifetime that no other code can name, used to make one store's keys unusable with another. This
   is `GhostCell`, and it is the most exotic thing in the day.

If the workshop is running to time you are reading this at about half past four, and both of these are
a victory lap rather than a load-bearing part of the day.
