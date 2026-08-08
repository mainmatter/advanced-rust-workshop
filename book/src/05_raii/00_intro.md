# RAII

`minidb` needs transactions: a group of changes that take effect together and can be taken back
together if something goes wrong halfway.

A transaction has to reach the store somehow, and with what the course has covered so far there is
exactly one way to arrange that. Give it the store:

```rust
pub struct Transaction {
    store: Store,
    undo: Vec<Undo>,
}

pub fn begin(self) -> Transaction
pub fn commit(self) -> Store
pub fn rollback(self) -> Store
```

Each change is applied to the store as it happens and the transaction records how to undo it, so
`rollback` can put everything back and `commit` keeps it. It works, and the first exercise is four
passing tests that prove it works and explain why nobody would ship it.

## What is wrong with it

**The store is inside the transaction.** Nothing outside can reach it, so `Transaction` has to grow its
own copy of every `Store` method a caller might want while a transaction is open. `get` is the first
one. It would not be the last.

**Every call site has to catch the store on the way out.**

```rust
let store = tx.commit();
```

every time, and the variable you started with is gone.

**The return types are bookkeeping.** `commit` and `rollback` both hand back a `Store`, which says
nothing about committing or rolling back. It is there because the ownership has to go somewhere.

**An early return loses everything.**

```rust
fn write_both(store: Store) -> Result<Store, Error> {
    let mut tx = store.begin();
    tx.insert(users.clone(), alice, Value::new("Alice"));

    let value = fetch_the_other_value()?;   // returns early

    tx.insert(users, bob, value);
    Ok(tx.commit())
}
```

The `?` drops the transaction, and the store is inside it, so the caller does not get half a database
back. It gets none.

## What we want instead

The store should stay where it is and lend itself to the transaction for a while, exclusively, so that
nothing else can touch it until the transaction finishes. That is `&mut Store`, and a struct that keeps
one needs a lifetime, which is the one piece of chapter 4 that chapter 4 had no reason to show you.

## Where this chapter goes

Five steps, and the last one is the interesting one:

1. **A borrowed store.** The transaction stops owning the store and borrows it instead, which is where
   a lifetime first goes on a struct.
2. **A drop guard.** Make the safe outcome automatic: an abandoned transaction rolls itself back.
3. **A drop bomb.** Safe is not the same as correct. Make the forgotten commit _say so_.
4. **The limits.** `Drop` is a strong default, not a guarantee. It is worth knowing exactly how strong.
5. **A closure API.** Stop asking callers to remember anything at all.

The pattern to watch for: each step moves the mistake earlier, from "silent corruption in production"
to "loud failure in a test" to "the code that could make the mistake does not exist".
