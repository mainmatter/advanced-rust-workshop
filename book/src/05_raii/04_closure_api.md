# Closure APIs

Look at what the last two sections actually achieved. A forgotten commit now rolls back and panics
loudly instead of corrupting data quietly. That is a large improvement, and the mistake is still there
to be made.

There is a move available that the guard cannot make: stop handing out the thing that can be misused.

```rust
impl Store {
    pub fn transaction<F, T, E>(&mut self, changes: F) -> Result<T, E>
    where
        F: FnOnce(&mut Transaction<'_>) -> Result<T, E>,
    {
        let mut tx = self.begin();

        let result = changes(&mut tx);

        if result.is_ok() {
            tx.commit();
        } else {
            tx.rollback();
        }

        result
    }
}
```

The caller never holds a `Transaction` they are responsible for:

```rust
store.transaction(|tx| {
    tx.insert(&users, &alice, Value::new("Alice"));
    let value = fetch_the_other_value()?;      // early return rolls back
    tx.insert(&users, &bob, value);
    Ok(())
})?;
```

The `?` still returns early. It now returns early from the _closure_, which is a value `transaction`
receives and acts on. The decision moved from the caller's discipline into your code, where it is
written once and tested once.

Notice that the drop bomb never fires through this path. The only code that could forget to commit is
the six lines you just wrote.

## The general shape

This is the same trick as `thread::scope`, `Vec::retain_mut`, `HashMap::entry` and every
`with_something` function you have ever called:

> Instead of giving the caller a resource and a rule, take a closure and apply the rule yourself.

It is the strongest of the three levels, and worth naming them explicitly:

| Level         | The mistake is        | Cost                    |
| ------------- | --------------------- | ----------------------- |
| Documentation | possible and silent   | free                    |
| Drop guard    | possible and harmless | a destructor            |
| Drop bomb     | possible and loud     | a destructor and a flag |
| Closure API   | not expressible       | flexibility             |

## The lifetime you did not write

There is an elided lifetime in that bound, and it does not behave like the ones in chapter 4:

```rust
F: FnOnce(&mut Transaction<'_>) -> Result<T, E>
```

In a function signature an elided lifetime becomes a parameter of the function. In an `Fn` bound it
does not: it gets its own binder, and the bound above means

```rust
F: for<'tx, 'store> FnOnce(&'tx mut Transaction<'store>) -> Result<T, E>
```

That is a **higher-ranked bound**, and it has to be one. `transaction` creates the `Transaction`
itself, inside its own body, so its lifetime is not something any caller could name. Try to make it a
parameter of the function and the compiler says so:

```rust
pub fn transaction<'tx, F, T, E>(&mut self, changes: F) -> Result<T, E>
where
    F: FnOnce(&'tx mut Transaction<'_>) -> Result<T, E>,
//  error[E0597]: `tx` does not live long enough
```

`for<'tx, 'store>` says the opposite of a parameter, and the opposite is what we mean: the callee
picks, and the closure has to cope with whatever it picks.

One trap, in case anyone tries to tidy it up. Collapsing the two binders into one type-checks as a
bound and then fails inside the body:

```rust
F: for<'tx> FnOnce(&'tx mut Transaction<'tx>) -> Result<T, E>,
//  error[E0505]: cannot move out of `tx` because it is borrowed
```

Tying the borrow of `tx` to `tx`'s own store lifetime keeps it borrowed for as long as it exists, so it
can never be moved into `commit`. `&mut T` is invariant in `T`, so the compiler cannot shrink its way
out. The two lifetimes have to stay separate, which is exactly what `'_` gives you for free.

## What it costs

That last column is not a rounding error, and the honest version of this advice includes it.

- **Borrows cannot escape.** The closure's return value cannot borrow from the transaction, because
  the transaction is gone by the time `transaction` returns. If a caller wants a reference to
  something inside, they have to clone it or restructure.
- **Control flow gets awkward.** `break` and `continue` do not cross a closure boundary, and `return`
  inside the closure returns from the closure. `?` works, which covers most real cases, but a loop
  that wants to abandon its transaction mid-iteration is now fighting you.
- **Composition suffers.** Two resources means two nested closures, and the rightward drift is real.
- **Async makes it worse.** A closure API that must accept an `async` block needs the higher-ranked
  bounds that Rust is still not good at expressing, and the error messages are terrible.

So the honest recommendation is **both**, which is what `minidb` now has: a closure API as the front
door for the ninety percent, and `begin` still there, still guarded, still armed, for the callers
whose control flow does not fit. `std` does the same thing: `thread::scope` for the common case,
`thread::spawn` when you need a handle.

What you should not do is offer only the raw version and document the rule. That is the level where
the mistake is silent, and this whole chapter is about not being there.
