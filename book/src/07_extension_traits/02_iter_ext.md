# Extending a trait

An extension trait does not have to extend a type. It can extend another trait, and that is how one
small crate adds forty methods to every iterator in the language:

```rust
pub trait IteratorExt: Iterator {
    fn collect_sorted(self) -> Vec<Self::Item>
    where
        Self: Sized,
        Self::Item: Ord,
    {
        let mut items = self.collect::<Vec<_>>();
        items.sort();
        items
    }
}

impl<I> IteratorExt for I where I: Iterator {}
```

Four pieces, each doing a specific job.

**The supertrait**, `: Iterator`, restricts the impl to iterators and gives the default body access to
`Iterator`'s methods. Without it, `self.collect()` does not exist.

**The default body** in the trait means the blanket impl can be empty. Anyone implementing
`IteratorExt` by hand gets the behaviour for free.

**The blanket impl** covers every iterator that exists, including types written after your crate. This
is the only way to add a method to types you have never heard of.

**`Self: Sized` on the method**, not on the trait. A method taking `self` by value needs a sized
receiver, but putting `Sized` on the trait itself would make `dyn IteratorExt` impossible. Keeping the
bound on the method is what lets `Iterator` itself be both object-safe and full of consuming adaptors.

## Method resolution, and how it bites

Rust looks for methods in a fixed order, and the first two rules explain most surprises:

1. **Inherent methods win over trait methods.** If `Store` has an inherent `export`, an extension
   trait's `export` will never be called on a `Store`. Adding an inherent method to a type in a later
   release can therefore silently steal calls from an extension trait, which is a real semver hazard.
2. **Then trait methods, but only from traits in scope**, walking `T`, `&T`, `&mut T` and then deref
   targets.

If two traits in scope both offer the method, neither wins:

```text
error[E0034]: multiple applicable items in scope
```

The caller's fix is fully qualified syntax:

```rust
IteratorExt::collect_sorted(iter)
```

which is correct, ugly, and not a problem the caller asked for. It is the strongest argument for
keeping extension traits small and specific: every method you add to every iterator in the program is
a name you have taken from everyone else.

## The judgement

Extension traits are a genuinely good tool with one failure mode: they are addictive. `Itertools` and
`anyhow`'s `Context` earn their place because they are focused, widely useful, and named so that a
reader can find where the method came from.

The test before adding one: **when a reader sees this method call, can they work out where it is
defined?** If the answer is "only by grepping the imports", the trait is too broad or the name is too
generic.
