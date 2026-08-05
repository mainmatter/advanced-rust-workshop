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

## Monomorphise the signature, not the body

`export_with` is generic, so the compiler stamps out one copy per format. What goes inside that copy
is your choice, and the obvious version duplicates far too much:

```rust
pub fn export_with<F>(&self, mut format: F) -> String
where
    F: Format,
{
    let mut buckets = self.buckets.iter().collect::<Vec<_>>();
    buckets.sort_by(|(left, _), (right, _)| left.cmp(right));

    for (bucket, values) in buckets {
        format.bucket(bucket);
        // sort this bucket's entries, walk them, call format.entry
    }

    format.finish()
}
```

Every line of that is copied per format, and every copy is identical except the three calls through
`format`. Two formats, two sorts, two loops. Add a third and you pay again.

Split it instead, and put the work behind `&mut dyn`:

```rust
pub fn export_with<F>(&self, mut format: F) -> String
where
    F: Format,
{
    self.render(&mut format)
}

pub fn export_into(&self, format: &mut dyn Format) -> String {
    self.render(format)
}

fn render(&self, format: &mut dyn Format) -> String {
    // the sorting and the walking, once
}
```

The generic part is now one line. `render` exists once in the binary however many formats there are,
both public methods are thin wrappers over it, and `export_into` gets the whole thing for free.

**When the generic parameter is there for the caller's ergonomics and the body is large, keep the
generic function thin and put the work behind `&mut dyn`.** You keep the signature you wanted and stop
paying for it per instantiation.

This is the cost worth managing. The previous section said the vtable call does not show up in a
profile, and it does not. Compile time and binary size do show up, and monomorphisation is what spends
them.
