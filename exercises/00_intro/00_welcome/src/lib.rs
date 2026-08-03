//! # Exercise
//!
//! Make the test at the bottom of this file pass.
//!
//! It is not a hard one. The point is to check that your toolchain works and that you know the loop:
//! run `wr`, read the failure, fix the code, run `wr` again.
//!
//! Exercises use `TODO` and `todo!()` markers to show you where to write code. You can also run the
//! tests directly with `cargo test` from this exercise's directory, which is what `wr` does for you.
//!
//! The tests are the specification: read them first, and never change them.

/// Reports whether you are ready to start.
pub fn ready() -> bool {
    todo!("this one really is a one-liner")
}

#[cfg(test)]
mod tests {
    use crate::ready;

    #[test]
    fn starting_block() {
        assert!(ready(), "Make `ready` return `true` and run `wr` again.");
    }
}
