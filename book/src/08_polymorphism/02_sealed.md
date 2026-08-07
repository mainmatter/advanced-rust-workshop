# Sealed traits

Since chapter 6, `Transaction` and `Writer` have carried a state parameter with nothing to say what
may go in it. `Transaction<'_, u32>` is a nameable type. Nobody can build one, so it has been a
curiosity rather than a hole, and we left it alone because the fix and the reason for the fix belong
together, here.

Writing the set down is one line per trait:

```rust
pub trait State {}
pub trait Section {}
```

and putting them to work is a bound on `Transaction<'store, S>` and `Writer<S>`. That much is
bookkeeping. The decision worth making is the next one, and it is the same decision for every public
trait you write: **is a third-party implementation of this something I want to work?**

`Format` is an **extension point**. Somebody else's crate should be able to add a format, and every
method they need to do that is public.

`State` and `Section` are not. `ReadOnly`, `ReadWrite`, `Root` and `InBucket` are the only members
that will ever make sense. A `Transaction<MyOwnState>` would satisfy the bound and have no methods at
all, because `insert` is defined on `Transaction<'_, ReadWrite>` and nowhere else.

Making a trait public and implementable is a promise. Sealing takes back half of it, and it is much
easier to do now than after the trait has been published open.

## The pattern

```rust
mod sealed {
    pub trait Sealed {}
}

pub trait State: sealed::Sealed {}

impl sealed::Sealed for ReadOnly {}
impl State for ReadOnly {}
```

The `sealed` module is private, so `sealed::Sealed` cannot be named outside this crate. A downstream
crate can still see `State`, still write `T: State` bounds, still call every method: it just cannot
write `impl State for MyType`, because it cannot implement the supertrait it would need.

The error a downstream user gets is honest, if not beautiful:

```text
error[E0277]: the trait bound `MyType: Sealed` is not satisfied
```

Adding a `# Sealed` note to the trait's documentation is worth the two lines it costs.

## What sealing buys

**Freedom to add methods.** A trait nobody outside can implement can grow a required method in a
minor release without breaking anyone. For an open trait, adding a required method is a breaking
change, and adding a defaulted one still risks colliding with an implementor's inherent method.

**Freedom to assume exhaustiveness.** If you know every implementor, you can match on them, add
blanket impls, and rely on invariants the trait itself does not express.

**A clearer contract.** The trait becomes documentation of a closed set rather than an invitation.

## What it does not buy, yet

None of that is cashed in here. `State` and `Section` are empty, nothing dispatches on them, and
`Transaction`'s private fields already stop an outside implementor from building a
`Transaction<MyOwnState>`. Sealing changes no code that exists, and the `compile_fail` test in the
exercise proves the mechanism works rather than that it matters.

That is the normal case, and it is still the right call, because the decision is irreversible in one
direction only. Sealing an open trait breaks every downstream implementor, so it can be done freely
just once, before publication. Unsealing is available forever and breaks nobody. What you are buying
is the option to add a method, or to assume the set is exhaustive, in a release you have not written
yet, at a price that only stays low until the crate ships.

## When not to seal

Sealing is a restriction on your users, so it needs a reason. Leave a trait open when you want people
to implement it: `Format` here, `Iterator`, `Read`, `serde::Serialize`.

The question to ask: **is a third-party implementation of this trait something I want to work?** If
yes, leave it open and accept that its signature is frozen. If the answer is "that would be
meaningless" or "that would break my invariants", seal it and say so in the docs.

`std` seals plenty: `SliceIndex`, `IsTerminal`, and every `os::unix` extension trait, `OsStrExt` and
`CommandExt` among them. All of them are closed sets that exist to be _used_ rather than extended.

## The nearby alternative

An enum is the other way to spell "a closed set", and it is often better:

```rust
pub enum Format { Ini, Csv }
```

An enum is closed by construction, exhaustively matchable, and needs no ceremony. It cannot be
extended by anyone, including you in a minor release, and it cannot carry per-variant behaviour
without a `match` in every method.

The rough division: **enum when the set is small and you dispatch on it; sealed trait when each member
carries its own behaviour or is used as a type parameter.** `State` and `Section` are type
parameters, so they have to be types, and sealing is the only way to close the set.
