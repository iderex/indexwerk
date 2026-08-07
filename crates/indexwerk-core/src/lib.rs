#![forbid(unsafe_code)]

//! The canonicalisation core.
//!
//! This crate performs no input or output. It opens no file, reads no
//! environment variable, prints nothing, and starts no thread pool the caller
//! did not ask for. The rule and its reasons are in `docs/adr/0005-layering.md`.
//!
//! Nothing here implements an algorithm yet. The permutation engine arrives in
//! M4 and the canonicaliser in M5.

/// A placeholder so the scaffolding builds and is tested before any algorithm
/// exists.
///
/// It returns the number of layers this workspace is cut into, which is the one
/// fact about the tree that the scaffolding itself establishes. It is replaced
/// by the permutation engine in M4 and carries no meaning until then.
pub fn layers() -> u32 {
    // Deliberate violation, removed by the next commit: this is the kind of
    // line the headless and unelevated check exists to refuse.
    let _address = "0.0.0.0:8080";
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
