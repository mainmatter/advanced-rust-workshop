# Ownership, borrowing and lifetimes

Most Rust programmers meet the borrow checker as an obstacle: the thing that rejects the code they
wanted to write. This chapter is about the other half of the deal, which is that the same machinery
enforces _your_ rules, for free, if you shape your signatures to ask for it. Before that pays off it is
worth being exact about what the rules are, because the next chapter is built on them.

## Two kinds of reference

```rust
&Store        // shared
&mut Store    // exclusive
```

A **shared reference** may be one of many. An **exclusive reference** is the only reference to that
value for as long as it exists.

Calling them "immutable" and "mutable" is the usual shorthand, and it is wrong in a way that matters:
`&Cell<T>`, `&Mutex<T>` and `&AtomicUsize` all let you change the value through a shared reference.
What `&mut` promises is not the right to write, it is the absence of anybody else. Permission to write
is what that promise buys.

## One rule

> Any number of shared references, or exactly one exclusive reference. Never both.

Aliasing XOR mutation, and almost every borrow error is this rule saying no. Holding a value borrowed
out of the store and then changing the store asks for both at once:

```rust
let alice = store.get(&users, &id);

store.insert(&users, &id, Value::new("Bob"));   // error[E0502]

println!("{alice:?}");
```

Two exclusive references are the same rule from the other side, and give `E0499`.

The rule is not bureaucracy. It is what makes iterator invalidation impossible, what makes data races
impossible without `unsafe`, and what lets the compiler assume that a value behind `&mut` cannot change
underneath it.

## A borrow lasts a region

Every reference is valid over a region of the code, and a **lifetime** is the name of that region. The
region is not the enclosing block: it ends at the reference's last use. Moving one line makes the
example above compile:

```rust
let alice = store.get(&users, &id).map(Value::as_str);
assert_eq!(alice, Some("Alice"));       // last use of the borrow

store.insert(&users, &id, Value::new("Bob"));   // fine
```

## Lexical, and then not

That is younger than the language. Before Rust 2018 a borrow really did run to the end of its block,
and the version above had to be written with an extra scope around the read purely to end the borrow
early. **Non-lexical lifetimes** replaced the block with the actual span of use, and a large class of
"the borrow checker is wrong" complaints disappeared with it.

One wrinkle is worth carrying into the next chapter: a value whose type implements `Drop` is _used_ at
the point where it is dropped, so its borrow does run to the end of the scope after all. That is
exactly what a guard wants.

## The ones you never write

You seldom spell a lifetime out, because most are inferred. `Store::get` is declared

```rust
pub fn get(&self, bucket: &Bucket, key: &Key) -> Option<&Value>
```

and means

```rust
pub fn get<'s, 'b, 'k>(&'s self, bucket: &'b Bucket, key: &'k Key) -> Option<&'s Value>
```

Three rules produce that, and they are the whole of **lifetime elision** for functions:

1. every elided input lifetime becomes its own parameter;
2. if there is exactly one input lifetime, every elided output gets it;
3. if one of the inputs is `&self` or `&mut self`, every elided output gets `self`'s lifetime instead.

There is a third spelling, `'_`, the **anonymous lifetime**: there is a borrow in this type and it is
not worth naming. You have written one already, in `Formatter<'_>` in chapter 3.

None of these rules apply to a struct that holds a reference. There you write the lifetime yourself,
which is the first thing chapter 5 does.

## The receiver is the API

Every method makes a claim about what the caller may do afterwards, and the receiver is where the claim
is written down:

| Receiver    | The caller keeps                           | Use it for                               |
| ----------- | ------------------------------------------ | ---------------------------------------- |
| `&self`     | shared access, others may read too         | queries                                  |
| `&mut self` | exclusive access for the borrow's lifetime | mutation, and anything needing isolation |
| `self`      | nothing                                    | one-shot operations, and conversions     |

Reaching for `self` is the move people forget. Any time an operation genuinely ends the life of a thing
(commit, close, finish, build, into_inner), taking `self` turns "please do not use this afterwards"
from a doc comment into a compiler error.

## Where this chapter goes

Two exercises, and they are the two halves of the same coin.

1. **Aliasing XOR mutability**, from the inside: the rule above is the same rule that stops you
   mutating a collection while walking it. Meeting it head-on and learning the standard ways through is
   most of what "fighting the borrow checker" turns out to be.
2. **Ownership in signatures**: `minidb` currently borrows things it immediately clones. That is a cost
   the caller pays and cannot see, and the fix is to say what you mean.
