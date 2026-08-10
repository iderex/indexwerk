#![forbid(unsafe_code)]

//! The canonicalisation core.
//!
//! This crate performs no input or output. It opens no file, reads no
//! environment variable, prints nothing, and starts no thread pool the caller
//! did not ask for. The rule and its reasons are in `docs/adr/0005-layering.md`.
//!
//! The data the algorithms work on: [`expression`] holds the index expression
//! model of #24, and [`rational`] the exact coefficient it carries.
//!
//! Of the algorithms themselves, the lowest layer is here. [`permutation`]
//! holds the permutation type, its sign and the total order the canonical form
//! is a minimum in, which is #20. What is above it is not: the base and strong
//! generating set, the orbit and stabiliser operations, and the double coset
//! search are the rest of M4 and M5, and none of them is in this crate yet.

pub mod expression;
pub mod permutation;
pub mod rational;

/// A placeholder so the scaffolding builds and is tested before any algorithm
/// exists.
///
/// It returns the number of layers this workspace is cut into, which is the one
/// fact about the tree that the scaffolding itself establishes. Nothing depends
/// on it and it carries no meaning; it goes when the canonicaliser has an entry
/// point for a caller to reach instead.
pub fn layers() -> u32 {
    3
}

#[cfg(test)]
mod tests {
    use super::layers;

    #[test]
    fn the_placeholder_is_callable_with_no_setup() {
        // The point of the assertion is not the number. It is that a test in
        // this crate needs no fixture, no file and no binding present, which is
        // what the layering record requires of the core.
        assert_eq!(layers(), 3);
    }
}
