# Drop guards

A **drop guard** is a value whose only job is to do something when it goes out of scope. `MutexGuard`
is the one everybody has met: it exists so that `unlock` cannot be forgotten, and it has no other
purpose.

Our transaction becomes one by implementing `Drop`:

```rust
impl Drop for Transaction<'_> {
    fn drop(&mut self) {
        self.undo_everything();
    }
}
```

That is the entire safety improvement. An early return, a `?`, a panic, an unhandled branch: all of
them now put the store back the way they found it.

## When drop runs

Worth being precise, because two of these surprise people:

- at the end of the scope where the value lives, in **reverse declaration order**;
- when a value is reassigned, on the old value;
- during **unwinding**, for everything on the stack between the panic and the `catch_unwind` (or the
  top of the thread);
- **not** when the value is moved: the new owner drops it instead.

The reverse order matters when guards depend on each other. Declare the outer resource first and the
inner one second, and they release in the order you would have written by hand.

## The `&mut self` problem

`Drop::drop` takes `&mut self`, never `self`. There is no way around it: the value has to be dropped
again after your code runs, so Rust cannot let you consume it.

Which means you cannot move fields out:

```rust
impl Drop for Transaction<'_> {
    fn drop(&mut self) {
        for undo in self.undo.into_iter() {   // error[E0507]: cannot move out of `self.undo`
```

The two standard ways out are both "leave something valid behind":

```rust
let undo = mem::take(&mut self.undo);        // leaves an empty Vec, needs Default
let value = self.value.take();               // Option::take, leaves None
```

`Option<T>` as a field purely so that `Drop` can take the `T` is a common shape. It is slightly
annoying to read, and it is the price of a destructor that owns something.

## Disarming

Here is the subtlety that catches people, and it is the second half of the exercise.

`commit` consumes `self`. So the transaction is dropped the moment `commit` returns, and the `Drop`
impl runs immediately afterwards, undoing everything that was just committed.

The fix is to make `Drop` able to tell that a decision was already made:

```rust
pub fn commit(mut self) {
    self.undo.clear();          // nothing left to undo, so drop is a no-op
}
```

Emptying the undo log is the cheapest version. A `bool` flag is the general one, and we will need it
in the next section. Either way the principle is the same: **a destructor that does real work needs a
way to be told the work is already done.**

The heavy-handed alternative is `mem::forget(self)`, which drops the value without running its
destructor. It works, and it is a bad habit: it skips the destructors of the _fields_ too, which for a
type holding a `Vec` means leaking memory. `ManuallyDrop` is the honest version of that idea when you
genuinely need it.

## Guards in the standard library

Once you know the shape you see it everywhere, and reading these is the fastest way to get a feel for
it:

| Guard                 | Runs on drop                   |
| --------------------- | ------------------------------ |
| `MutexGuard`          | unlocks the mutex              |
| `File`                | closes the descriptor          |
| `Box`, `Vec`          | frees the allocation           |
| `JoinHandle` (scoped) | joins the thread               |
| `BufWriter`           | flushes, and ignores the error |

That last row is the one to remember when you write your own. `Drop::drop` returns `()`, so a
destructor cannot report a failure. `BufWriter` flushes on drop and swallows any error, which is why
its documentation tells you to call `flush()` yourself if you care whether the bytes arrived.

**A destructor is not a place to do fallible work.** If the cleanup can fail in a way the caller
should know about, give them an explicit method that returns a `Result`, and keep the destructor as
the fallback that stops things being worse.
