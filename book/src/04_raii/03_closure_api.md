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
        let mut txn = self.begin();

        match changes(&mut txn) {
            Ok(value) => {
                txn.commit();
                Ok(value)
            }
            Err(error) => {
                txn.rollback();
                Err(error)
            }
        }
    }
}
```

The caller never holds a `Transaction` they are responsible for:

```rust
store.transaction(|txn| {
    txn.insert(&users, &alice, Value::new("Alice"));
    let value = fetch_the_other_value()?;      // early return rolls back
    txn.insert(&users, &bob, value);
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
