# RAII

`minidb` has transactions now:

```rust
let mut tx = store.begin();
tx.insert(&users, &id, Value::new("Alice"));
tx.commit();
```

Each change is applied to the store as it happens, and the transaction records how to undo it, so
`rollback` can put everything back.

There is one obvious way to get this wrong, and everybody does:

```rust
fn write_both(store: &mut Store) -> Result<(), Error> {
    let mut tx = store.begin();
    tx.insert(&users, &alice, Value::new("Alice"))?;

    let value = fetch_the_other_value()?;   // returns early

    tx.insert(&users, &bob, value);
    tx.commit();
    Ok(())
}
```

The early return skips the commit, so the transaction is dropped where it stands and half the work is
now permanent. No error, no warning: the type system watched the whole thing happen and said nothing.

Note how little the mistake looks like a mistake. There is no forgotten `close()`, no unbalanced
`unlock()`, just a `?` in the middle of a function, which is the most ordinary thing in Rust.

## The idea

**Resource Acquisition Is Initialisation** is a terrible name for a good idea. The idea is:

> Tie the cleanup to a value, and let the compiler run it when the value goes away.

You do not have to remember. You cannot forget on the error path, because the error path drops your
values too. You cannot forget on the panic path either, because unwinding drops them as well.

Rust leans on this harder than any mainstream language, because ownership tells the compiler exactly
when each value dies. `Box` frees, `File` closes, `MutexGuard` unlocks, `JoinHandle` waits. None of
those need a `finally` block, and none of them can be skipped by an early return.

## Where this chapter goes

Four steps, and the last one is the interesting one:

1. **A drop guard.** Make the safe outcome automatic: an abandoned transaction rolls itself back.
2. **A drop bomb.** Safe is not the same as correct. Make the forgotten commit _say so_.
3. **The limits.** `Drop` is a strong default, not a guarantee. It is worth knowing exactly how strong.
4. **A closure API.** Stop asking callers to remember anything at all.

The pattern to watch for: each step moves the mistake earlier, from "silent corruption in production"
to "loud failure in a test" to "the code that could make the mistake does not exist".
