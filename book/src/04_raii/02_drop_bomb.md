# Drop bombs and the limits of Drop

The guard made the safe outcome automatic. It also made a bug invisible: a transaction somebody forgot
to commit now behaves exactly like one they meant to abandon. The data is safe and the program is
still wrong.

A **drop bomb** is a guard that panics when it is dropped without an explicit decision:

```rust
impl Drop for Transaction<'_> {
    fn drop(&mut self) {
        if self.finished {
            return;
        }

        self.undo_everything();

        if !thread::panicking() {
            panic!("transaction dropped while neither committed nor rolled back");
        }
    }
}
```

The bomb turns "silent corruption in production" into "loud failure in the test suite", which is the
trade every time you can get it.

## The rule you cannot skip

**A panic during unwinding aborts the process.** Not a nicer panic, not a caught panic: `abort`, with
no unwinding, no destructors, and no message beyond a terse note about panicking in a destructor.

That is why the `thread::panicking()` check is there. Without it, any real failure inside a
transaction scope, an `assert!` in a test, a genuine bug, gets replaced by a hard abort that tells you
nothing about the original problem. You lose the actual error and gain a dead process.

So the rule for any destructor that can panic:

```rust
if !thread::panicking() {
    panic!("...");
}
```

Some crates go further and only arm the bomb in debug builds, on the grounds that aborting a
production process over a bookkeeping mistake is worse than the mistake. That is a judgement call
about who is running the code.

## When not to arm it

[`sqlx`](https://docs.rs/sqlx) is the counterexample worth knowing, because it is the same domain. Its
`Transaction` implements `Drop` and rolls back silently: if neither `commit` nor `rollback` is called
before the transaction goes out of scope, the changes are undone and nothing is said. A plain guard,
and a deliberate refusal to take the next step.

The reason is that dropping a transaction _means_ something there. Abandoning one on the error path is
the idiom: you write `?` after each query, and the abandonment is the abort. A bomb would fire on
correct code. Async sharpens the same point well beyond databases, because dropping a future is an
ordinary event. A timeout fires, a `select!` branch loses, a request is cancelled, and everything in
that future's state is dropped without anyone having done anything wrong. A destructor cannot tell
cancellation from forgetting, so an armed bomb would turn every timeout into a process abort.

`sqlx` does not even roll back in `drop`, for that matter. `Drop` is synchronous and a rollback needs
I/O, so it marks the connection and the `ROLLBACK` goes out when that connection is next used or
returned to the pool. A guard that is already best-effort and deferred is the wrong place to assert
anything.

So the test is: **arm the bomb only where dropping without a decision has no legitimate meaning.**

`rust-analyzer`'s parser passes it. Its `Marker` must be completed or abandoned, forgetting is
unambiguously a bug, it is all one codebase, and the panic lands in a developer's own test run rather
than in a user's server. That is what matklad's [`drop_bomb`](https://docs.rs/drop_bomb) crate is for,
and it is where the name comes from.

`minidb` passes it too, but only because of where this chapter is going. Arming the bomb changes what
the `?` in `write_both` does: the early return we opened with now panics instead of quietly rolling
back. If `begin` were the interface we shipped, that would be a hard sell. It is not. The next section
puts a closure API in front of it that makes the decision itself, which leaves the bomb armed for the
callers who need the raw form and off the path everybody else takes.

## Drop is a strong default, not a guarantee

It is genuinely possible for a destructor never to run, and leaking is **safe** in Rust: it is not
`unsafe`, and no rule in the language promises `drop` will happen.

The ways it does not run:

- **`mem::forget(value)`**, which exists precisely to skip it, and `ManuallyDrop<T>`, its typed form.
- **`Box::leak`**, and anything else that deliberately gives up ownership for a `'static` reference.
- **Reference cycles.** Two `Rc`s pointing at each other keep the count above zero forever. Nothing in
  the language stops you.
- **`process::exit` and `abort`**, which do not unwind at all.
- **`panic = "abort"`**, a profile setting that turns every panic into an abort and skips every
  destructor on the way out.
- **Leaked threads.** A detached thread's stack is never unwound if the process ends first.

This was settled deliberately, in a long argument that has its own name: leaking was going to be
possible via `Rc` cycles no matter what the API said, so `mem::forget` became safe rather than
pretending otherwise.

The practical consequence: **you cannot use `Drop` to enforce a safety invariant**. It is a
convenience and a very good default, not a proof. If correctness depends on cleanup happening, the
cleanup has to be on the only path that reaches the result, which is what the next section is about.

## Scope guards

The general form of a drop guard is worth having in your pocket:

```rust
pub struct ScopeGuard<F: FnOnce()> {
    action: Option<F>,
}

impl<F> Drop for ScopeGuard<F>
where
    F: FnOnce(),
{
    fn drop(&mut self) {
        if let Some(action) = self.action.take() {
            action();
        }
    }
}
```

An `Option` so the closure can be moved out of `&mut self`, and a `dismiss` method that takes it
without calling it. This is `defer` from Go, except that it fires at end of scope rather than end of
function, and it composes properly with early returns. The
[`scopeguard`](https://docs.rs/scopeguard) crate is the version you should actually use, and reading
its source takes five minutes.
