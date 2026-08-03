# Welcome

Welcome to Mainmatter's **Advanced Rust** workshop!

You know Rust. You can read a lifetime annotation without flinching, you have shipped something real,
and `cargo clippy` rarely surprises you any more. This course is about the next step: designing Rust
APIs that other people, including future you, cannot get wrong.

The premise is simple. Every rule your domain has is enforced somewhere: in a comment, in a code
review, in a runtime check, or in the type system. The further left you push it, the cheaper it gets.
Rust gives you an unusually powerful set of tools for pushing rules all the way into the compiler, and
most Rust code uses a small fraction of them.

## What you will build

Rather than a series of unrelated puzzles, you will build one library from nothing: **`minidb`**, a small
embedded key-value store, in the spirit of [`redb`](https://docs.rs/redb) or [`sled`](https://docs.rs/sled).

It starts as the kind of code anyone would write in an afternoon: `HashMap`s, `&str` parameters, and
`Option` everywhere. By the end of the day, it will be a library where forgetting to commit a
transaction is a compile-time error, where a key from one store cannot be used with another, and where
the only way to hold a value you should not have is to write `unsafe`.

Each exercise is a complete, standalone copy of the library at that point in its evolution. You never
have to carry a broken state forward.

## Methodology

This is a hands-on workshop. Expect to spend at least half the day writing code.

Exercises are test-driven: each one ships a set of tests that describe the API you are supposed to
build, and your job is to make them pass. Some tests are `compile_fail` doctests, which assert that
certain code must **not** compile. Those are the interesting ones: in this course, a compiler error is
frequently the feature.

Exercises include `TODO` and `todo!()` markers to draw your attention to the lines where you need to
write code. Sometimes a single line is enough, sometimes you will need to reshape a whole type.

> ⚠️ **Do not modify the tests.** They are the specification. Change the code under test, not the test.

If you get stuck for more than ten minutes, grab a trainer. We are here to help. You can also find
solutions to all exercises in the `solutions` branch of this repository.

## Setup

You need a recent stable Rust toolchain:

```bash
rustup update stable
```

Clone the repository and create a branch to work on:

```bash
git clone https://github.com/mainmatter/advanced-rust-workshop
cd advanced-rust-workshop
git checkout -b my-solutions
```

Then install the workshop runner, the tool that walks you through the exercises:

```bash
cargo install --locked workshop-runner
```

## The workflow

From the root of the repository, run:

```bash
wr
```

`wr` finds the first exercise you have not solved yet, compiles it, runs its tests, and either
congratulates you or shows you what went wrong. It will not let you move on until the current exercise
compiles and passes. Solve it, run `wr` again, and it opens the next one.

That is the whole loop. Let's make sure it works.
