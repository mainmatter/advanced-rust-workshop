# Polymorphism

`Store::export` renders one format, and the format is welded into the method. The next request is
always CSV, or JSON, or whatever the reporting team standardised on this quarter.

So we need to be polymorphic over "some format". Rust offers two mechanisms, and choosing between
them is one of the few API decisions that is genuinely hard to reverse:

```rust
fn export_with<F: Format>(&self, format: F) -> String        // static dispatch
fn export_into(&self, format: &mut dyn Format) -> String     // dynamic dispatch
```

They look almost the same at the call site and are entirely different underneath.

## Static dispatch

`export_with` is not one function. The compiler generates a separate copy for every `F` you call it
with, each one knowing exactly which `Format` it is talking to. That is **monomorphisation**, and it
is why generic Rust has no dispatch cost: after inlining, `format.entry(..)` is a direct call, often
no call at all.

The costs are real but indirect: compile time, binary size, and the fact that `Vec<F>` can hold only
one kind of format.

## Dynamic dispatch

`&mut dyn Format` is a **fat pointer**: one pointer to the value, one to a vtable of function
pointers. There is one copy of `export_into` in the binary, and the call goes through the vtable, so
it cannot be inlined.

What you get for that is the thing static dispatch cannot do: a `Vec<Box<dyn Format>>` holding three
different formats, a format chosen from a config file at run time, a plugin loaded from a shared
library.

## The part that surprises people

Not every trait can be a `dyn`. **Dyn compatibility** (previously called object safety) is a property
of the trait, and it constrains how you write the trait even if nobody ever writes `dyn Format`.

The next section builds `Format` under that constraint, and the constraint is visible in the
signature:

```rust
fn finish(&mut self) -> String;      // what we have to write
fn finish(self) -> String;           // what reads better, and is not dyn-compatible
```

Then the last section goes the other way, and takes the ability to implement a trait away from
everybody else on purpose.
