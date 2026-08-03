# The borrow checker as an enforcement engine

Most Rust programmers meet the borrow checker as an obstacle: the thing that rejects the code they
wanted to write. This chapter is about the other half of the deal, which is that the same machinery
enforces _your_ rules, for free, if you shape your signatures to ask for it.

`minidb` already has transaction isolation. Nobody wrote it.

## Three guarantees nobody paid for

```rust
pub fn begin(&mut self) -> Transaction<'_>   // exclusive access, for as long as the transaction lives
pub fn commit(self)                          // consumes the transaction
```

Two signatures, three rules the compiler now enforces:

| Rule                                 | Enforced by                                     | Error   |
| ------------------------------------ | ----------------------------------------------- | ------- |
| Only one transaction at a time       | `&mut self`, held by the returned `Transaction` | `E0499` |
| No reads while a transaction is open | the same exclusive borrow                       | `E0502` |
| A transaction is finished once       | `commit(self)`                                  | `E0382` |

A database usually buys the first two with a lock, and pays for them at runtime, forever. Here they
cost nothing at runtime, cannot be forgotten, and cannot be worked around without changing the
signatures.

The third is worth a second look. `commit(self)` does not merely discourage a double commit: it makes
the second call impossible, because the value is gone. That is a **single-use value**, also called a
linear type, and it is the cheapest state machine in Rust.

## The receiver is the API

Every method makes a claim about what the caller may do afterwards, and the receiver is where the
claim is written down:

| Receiver    | The caller keeps                           | Use it for                               |
| ----------- | ------------------------------------------ | ---------------------------------------- |
| `&self`     | shared access, others may read too         | queries                                  |
| `&mut self` | exclusive access for the borrow's lifetime | mutation, and anything needing isolation |
| `self`      | nothing                                    | one-shot operations, and conversions     |

Reaching for `self` is the move people forget. Any time an operation genuinely ends the life of a
thing (commit, close, finish, build, into_inner), taking `self` turns "please do not use this
afterwards" from a doc comment into a compiler error.

## Where this chapter goes

Two exercises, and they are the two halves of the same coin.

1. **Aliasing XOR mutability**, from the inside: the rule that makes the guarantees above possible is
   the same rule that stops you mutating a collection while walking it. Meeting it head-on and
   learning the standard ways through is most of what "fighting the borrow checker" turns out to be.
2. **Ownership in signatures**: `minidb` currently borrows things it immediately clones. That is a
   cost the caller pays and cannot see, and the fix is to say what you mean.
