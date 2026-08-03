# Names and docs

Before any of the clever type-level machinery, an API is a list of names and a page of prose. Both are
read far more often than the code behind them, and both are the parts nobody budgets time for.

This chapter is short and deliberately unglamorous. It is here because the rest of the day assumes it:
a `Key` type is only an improvement if it is called `Key`, and a `parse` method that returns `Result`
is only usable if its failure modes are written down somewhere.

## The starting point

```rust
store.set_value("users", "42", "Alice");
store.get_value("users", "42");
store.is_has_bucket("users");
store.get_count();
store.as_map();
```

Every one of those works. Every one of them is called something you would have to look up, because each
name was invented in isolation rather than borrowed from the vocabulary Rust programmers already know.

## Two ideas

Everything in this chapter follows from two claims.

**A name is a promise about cost and ownership.** `as_bytes` promises a cheap borrow. `to_owned`
admits an allocation. `into_bytes` warns that the receiver is consumed. A Rust programmer reading
`as_map()` has already concluded that it is free, and will call it in a loop.

**A doc comment is for the caller, and the caller cannot see the body.** So it documents what the
function is for, what it promises, and what it does to them on a bad day. Not how it works: they can
read that, and if they cannot, the comment will be wrong within a release anyway.
