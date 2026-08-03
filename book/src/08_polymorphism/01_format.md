# Generics and `dyn`

## Dyn compatibility

A trait can be made into a trait object only if every method can be called through a fat pointer,
which means the compiler must be able to build a vtable entry for it. The rules that matter in
practice:

| Not allowed in a `dyn` trait         | Why                                                    |
| ------------------------------------ | ------------------------------------------------------ |
| `fn finish(self)`                    | a trait object is unsized, so there is nothing to move |
| `fn write<W: Write>(&self, w: W)`    | one vtable slot cannot hold every instantiation        |
| `fn entries(&self) -> impl Iterator` | the return type differs per implementor                |
| `fn make() -> Self`                  | no receiver, and `Self` is unknown                     |
| `Self: Sized` on the trait           | says "never a trait object" outright                   |

The escape hatch for the first four is `where Self: Sized` **on the method**: such a method is
excluded from the vtable and remains callable on concrete types. This is how `Iterator` is
dyn-compatible while still having `map`, `collect` and forty other consuming, generic adaptors.

So `Format` is written like this:

```rust
pub trait Format {
    fn bucket(&mut self, bucket: &Bucket);
    fn entry(&mut self, key: &Key, value: &Value);
    fn finish(&mut self) -> String;
}
```

`finish(&mut self)` rather than `finish(self)`, and the implementations end up doing
`mem::take(&mut self.output)`. That is a small ugliness in exchange for `Box<dyn Format>` being
possible, and it is a decision you make once, at the trait, for all time.

## Choosing

Both versions can coexist, and in `minidb` they do, but the default matters. A checklist that
resolves most cases:

**Reach for generics when** the set of types is known at compile time, when the calls are hot, when
the trait has generic methods or consuming methods you do not want to contort, or when you want the
strongest possible inlining.

**Reach for `dyn` when** the type is chosen at run time, when you need a heterogeneous collection,
when the extra copies would bloat compile times for no benefit, or when the trait is a plugin
boundary.

Two rules of thumb from the standard library and its ecosystem:

- **Take `impl Trait` in argument position by default.** It is the generic version with less
  ceremony, and callers cannot tell the difference.
- **Prefer `&dyn Trait` to `Box<dyn Trait>`** where the value does not need to be owned. `Box`
  allocates; a reference does not.

## The cost, measured honestly

Dynamic dispatch costs an indirect call and the loss of inlining, and that is usually irrelevant. The
real difference is what the two enable, not what they cost:

- static: no `Vec<F>` with mixed formats, but zero overhead and monomorphised errors that mention your
  concrete type;
- dynamic: one function in the binary, run-time choice, and a trait you have to keep dyn-compatible
  forever.

The performance argument is the one people reach for first and it is usually the least important. In
a function that walks a `HashMap` and pushes to a `String`, the vtable call does not appear in a
profile.

## Generic over the wrong thing

One trap worth naming. This is generic:

```rust
fn export_with<F: Format>(&self, format: F) -> String
```

and this is not:

```rust
fn render(&self, format: &mut dyn Format) -> String
```

`export_with` delegates to `render`, so the monomorphised code is tiny: one wrapper per format,
sharing one real implementation. That is a useful pattern when the generic surface is for ergonomics
and the body is large: keep the generic function thin and put the work in a `dyn` function. It cuts
compile time and binary size while leaving the nice signature in place.
