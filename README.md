# Advanced Rust

> Make illegal states unrepresentable

Rust's type system is not a formality you satisfy on the way to a running program: it is the cheapest
place to encode the rules of your domain. This workshop is about using it that way.

You will work through a series of test-driven exercises, building `minidb`, a small embedded key-value
store, one API design decision at a time. By the end you will have a library whose misuse is a
compile-time error rather than a production incident.

This workshop is designed for people who are comfortable writing Rust and want to get better at
designing Rust APIs that other people have to live with.

> [!NOTE]
> This workshop has been written by [Mainmatter](https://mainmatter.com/rust-consulting/).\
> It's one of the trainings in [our portfolio of Rust workshops](https://mainmatter.com/services/workshops/rust/).\
> Check out our [landing page](https://mainmatter.com/rust-consulting/) if you're looking for Rust consulting or training!

## Getting started

Open the companion book for this course in your browser. Follow the instructions there to get started.

## Requirements

- **Rust** (follow instructions [here](https://www.rust-lang.org/tools/install)).\
  If Rust is already installed on your system, make sure you are running on the latest compiler version (`cargo --version`).\
  If not, update using `rustup update` (or another appropriate command depending on how you installed Rust on your system).
- _(Optional)_ An IDE with Rust autocompletion support.
  We recommend one of the following:
  - [RustRover](https://www.jetbrains.com/rust/);
  - [Visual Studio Code](https://code.visualstudio.com) with the [`rust-analyzer`](https://marketplace.visualstudio.com/items?itemName=matklad.rust-analyzer) extension.

## Solutions

You can find the solutions to the exercises in the `solutions` branch of this repository.

## References

Throughout the workshop, the following resources might turn out to be useful:

- [Rust Book](https://doc.rust-lang.org/book/)
- [Rust documentation](https://doc.rust-lang.org/std/) (you can also open the documentation offline with `rustup doc`!)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [The `std::marker` module](https://doc.rust-lang.org/std/marker/)

# License

Copyright © 2026- Mainmatter GmbH (https://mainmatter.com), released under the
[Creative Commons Attribution-NonCommercial 4.0 International license](https://creativecommons.org/licenses/by-nc/4.0/).
