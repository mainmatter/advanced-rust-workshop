# Branded lifetimes

A `Key` parsed anywhere works with any `Store`. In a system with one store that is fine. In a system
with several, using store A with a key you built while thinking about store B compiles, returns `None`
or the wrong row, and is a real class of bug.

The types cannot tell the stores apart, because there is only one `Store` type. What we need is a way
to make each _value_ have its own type, and Rust has exactly one thing that is fresh at every call
site: a lifetime.

## The trick

```rust
pub struct Scoped<'brand> {
    store: Store,
    _brand: PhantomData<fn(&'brand ()) -> &'brand ()>,
}

pub fn scope<F, R>(changes: F) -> R
where
    F: for<'brand> FnOnce(Scoped<'brand>) -> R,
{
    changes(Scoped { store: Store::new(), _brand: PhantomData })
}
```

Two pieces, neither guessable, both essential.

**`for<'brand>`** is a higher-ranked bound: the closure must work for _every_ lifetime, so it cannot
be written to expect a particular one. Each call to `scope` therefore hands the closure a fresh,
anonymous lifetime that no other call, and no code outside, can name or unify with.

**`PhantomData<fn(&'brand ()) -> &'brand ()>`** makes `'brand` **invariant**. Without it the lifetime
would be covariant, so a longer brand could be shortened to match a shorter one, two brands would
happily unify, and the whole thing would silently do nothing.

That is why the marker is a function type. Function types are contravariant in their arguments and
covariant in their return, and the only lifetime that satisfies both at once is the exact one.
`fn(T) -> T` is the standard spelling of "invariant in `T`", and unlike `&mut T` it keeps the type
`Send` and `Sync`.

## Variance, briefly

Variance is the rule for when one type may substitute for another:

|               | Meaning                         | Example                           |
| ------------- | ------------------------------- | --------------------------------- |
| covariant     | `Foo<'long>` is a `Foo<'short>` | `&'a T`, `Box<T>`                 |
| contravariant | `Foo<'short>` is a `Foo<'long>` | the argument of `fn(T)`           |
| invariant     | neither                         | `&mut T`, `Cell<T>`, `fn(T) -> T` |

Covariance is what lets you pass a `&'static str` to a function wanting `&'a str`, and you have relied
on it all day without noticing. Invariance is what you reach for when a lifetime is being used as an
identity rather than as a duration, which is exactly what a brand is.

## What it buys

The compile error is the product:

```rust
let stolen = scope(|store| store.key("42").unwrap());

scope(|store| store.get(&users, &stolen));      // does not compile
```

Beyond stopping mix-ups, this is the foundation of a family of zero-cost APIs. If a library can prove
an index was checked against a particular collection, it can hand out an accessor that skips the
bounds check without `unsafe` at the call site. That is what `GhostCell` and the `generativity` crate
are for, and it is how `indexing`-style crates offer checked-once, used-many access.

## Should you use this?

Usually not, and it is worth being blunt about it at the end of a long day.

The costs are heavy: every API has to be inside a closure, the lifetime is contagious across every
type that touches a branded value, error messages become genuinely hard to read, and the technique is
unfamiliar enough that a reviewer will need the explanation you just read.

Reach for it when mixing up instances is both **easy** and **silently wrong**, and when the API is
narrow enough that the closure ceremony is a one-time cost. Arena indices are the honest use case.
Two stores in an application are usually better served by naming the variables carefully.

The reason it is the last thing in this workshop is not that it is the most useful. It is that it is
the furthest point on the line the whole day has been walking: state in the types, permission in the
types, protocol in the types, and finally identity in the types.
