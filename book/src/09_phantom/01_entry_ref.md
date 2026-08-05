# Lifetimes for things that are not references

Here is a handle that remembers where a value lives:

```rust
pub struct EntryRef<'store> {
    bucket: Bucket,
    key: Key,
    _store: PhantomData<&'store Store>,
}
```

Look at what it holds: two owned names, both `'static`, and no references at all. Left to itself, this
type could outlive the store, be sent to another thread, and be read against a store where the entry
has since been deleted. The compiler would have nothing to say, because nothing in the data says
otherwise.

`PhantomData<&'store Store>` says otherwise. It makes the handle behave, for the borrow checker,
exactly like a shared reference to the store, which is what it morally is. Every ordinary rule then
applies for free:

- the handle cannot outlive the store;
- the store cannot be mutated while the handle exists;
- several handles can coexist, because shared borrows do.

None of that is code you wrote. It is one field of a type that occupies no space.

## Where else this shows up

Once you know the shape you find it in every wrapper over a foreign resource: an index into an arena,
a row id from a database handle, a `slab` key, a GPU buffer handle. Anywhere the value is a plain
integer that is only meaningful relative to something else.

## Which `PhantomData` to write

The choice affects variance and auto traits, so it is worth being deliberate rather than copying:

| Marker                    | Means                                                         |
| ------------------------- | ------------------------------------------------------------- |
| `PhantomData<T>`          | I own a `T`, drop like it, and inherit its auto traits        |
| `PhantomData<&'a T>`      | I borrow a `T` for `'a`, shared                               |
| `PhantomData<&'a mut T>`  | I borrow a `T` for `'a`, exclusive, and I am invariant in `T` |
| `PhantomData<*const T>`   | I am not `Send` and not `Sync`                                |
| `PhantomData<fn(T) -> T>` | I am invariant in `T`, and I stay `Send` and `Sync`           |

The last one looks bizarre and is the workhorse of the next section.

For our handle, `PhantomData<&'store Store>` is right: shared, tied to the store, and it keeps the
handle `Send` if the store is.
