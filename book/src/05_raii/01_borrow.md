# A borrowed store

```rust
pub struct Transaction<'store> {
    store: &'store mut Store,
    undo: Vec<Undo>,
}

pub fn begin(&mut self) -> Transaction<'_>
pub fn commit(self)
pub fn rollback(self)
```

Every complaint from the previous section goes away at once. The store stays where the caller put it,
`commit` and `rollback` have nothing to hand back, and `Transaction::get` can be deleted because
nothing was taken away in the first place.

## The lifetime goes on the struct

`'store` is a lifetime parameter, and it is the first one in this course you have to write yourself.
Elision covers functions; it has nothing to say about structs, because there is no call site to infer
from. A struct that holds a reference must declare the region that reference is valid over, and then
the struct itself is only valid over that region.

Which is exactly the guarantee we want:

```rust
let tx = {
    let mut store = Store::new();
    store.begin()          // error[E0597]: `store` does not live long enough
};
```

A `Transaction<'store>` can never outlive the `Store` it came from, and nobody had to write a check.

## `'_` everywhere else

`begin` needs no name for it:

```rust
pub fn begin(&mut self) -> Transaction<'_>
```

`&mut self` is the only input lifetime, so elision rule three gives it to the elided output. The `'_`
is not doing the inference, it is announcing it: there is a borrow inside this type, and I am not
naming it. Leaving it out entirely still compiles, and the compiler will tell you off:

```text
warning: hiding a lifetime that's elided elsewhere is confusing
```

The impl header takes the same `'_`, because none of the methods care which region it is:

```rust
impl Transaction<'_> { .. }
```

## Three guarantees nobody paid for

Look at what those two signatures have quietly bought:

| Rule                                 | Enforced by                                     | Error   |
| ------------------------------------ | ----------------------------------------------- | ------- |
| Only one transaction at a time       | `&mut self`, held by the returned `Transaction` | `E0499` |
| No reads while a transaction is open | the same exclusive borrow                       | `E0502` |
| A transaction is finished once       | `commit(self)`                                  | `E0382` |

A database usually buys the first two with a lock and pays for them at runtime, forever. Here they cost
nothing at runtime, cannot be forgotten, and cannot be worked around without changing the signatures.

The third is worth a second look. `commit(self)` does not merely discourage a double commit: it makes
the second call impossible, because the value is gone. That is a **single-use value**, also called a
linear type, and it is the cheapest state machine in Rust.

Nobody wrote a line of code about transaction isolation. It fell out of choosing `&mut self` over
`self`, and `self` over `&self`, one line each.
