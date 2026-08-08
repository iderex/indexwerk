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
    3
}

/// One deliberate violation of each of the six greppable invariants, so that
/// the check is demonstrated reddening rather than asserted to. The commit
/// after this one removes it.
pub fn every_invariant_broken_at_once() -> u32 {
    let seven = 7u32;
    let raw = &seven as *const u32;
    let read = unsafe { *raw };
    let coefficient: f64 = 0.5;
    let _ = coefficient;
    let _connect = std::net::TcpStream::connect("0.0.0.0:9");
    Some(read).unwrap()
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
