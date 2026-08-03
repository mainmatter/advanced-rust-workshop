# Naming conventions

Rust's naming conventions are not a style preference. They are a compression scheme: a caller who knows
them can predict the cost, the ownership and the failure mode of a method from its name alone, without
opening the documentation.

The [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/naming.html) list them all. What
follows is the subset that carries real information.

## Cost and ownership: `as_`, `to_`, `into_`

The most valuable convention in the language, and the most frequently broken.

| Prefix  | Cost                           | Receiver | Example              |
| ------- | ------------------------------ | -------- | -------------------- |
| `as_`   | free, a view of the same bytes | borrowed | `str::as_bytes`      |
| `to_`   | allocates or computes          | borrowed | `str::to_owned`      |
| `into_` | free or cheap, but consuming   | owned    | `String::into_bytes` |

A method called `as_map` that clones a `HashMap` is not a naming quibble. It is a lie about
performance, and it will be called inside a loop by someone who trusted it.

## Constructors

- `new` is the obvious constructor, and takes no options nobody would expect.
- `with_capacity`, `with_config`: a constructor with one salient parameter.
- `from_*` for conversions that cannot fail, `try_from`/`parse` for conversions that can.
- `Default::default` when "empty" is meaningful, and prefer it to `new()` taking no arguments only
  when a default genuinely exists.

## Borrow the standard library's vocabulary

Collections have already settled this, and every Rust programmer has already learned it:

| Operation                       | The word                   |
| ------------------------------- | -------------------------- |
| add, returning what it replaced | `insert`                   |
| take out, returning it          | `remove`                   |
| look up, borrowed               | `get`                      |
| look up, mutable                | `get_mut`                  |
| how many                        | `len`                      |
| is it zero                      | `is_empty`                 |
| is this in here                 | `contains`, `contains_key` |

`set_value`, `delete`, `size` and `has_key` all work and all cost the reader a lookup. Reach for the
word the standard library already uses, even when your own word is marginally more accurate.

## Prefixes that carry nothing

- **`get_`** is noise. `store.get_count()` says "get" twice, once in the verb and once in the fact that
  it is a method returning a value. The standard library uses `get` alone, for lookups that can fail.
- **`is_`** is for adjectives: `is_empty`, `is_ascii`. Possession is `contains_` or `has_`. `is_has_bucket`
  is the sound of two conventions colliding.
- **`_mut`** pairs with a borrowed getter of the same name: `get`/`get_mut`, `iter`/`iter_mut`. If you
  have a `_mut` method with no partner, one of the two is misnamed.

## Where there is a `len`, there is an `is_empty`

A small one, and Clippy will tell you off for it. If your type has a `len`, callers will write
`x.len() == 0`, which is both noisier and, for lazy or computed collections, potentially slower. Give
them `is_empty`.

## The test

For each name, ask: could a competent Rust programmer who has not read this file guess what it returns,
whether it allocates, and whether it takes ownership? If the answer is no for any of the three, the
name is doing less work than it could.
