#![forbid(unsafe_code)]

//! The Python package.
//!
//! It sits on the C interface like any other consumer and has no privileged
//! access to the core. `docs/adr/0005-layering.md` is where that is decided.
//!
//! The interpreter is not present yet. The extension module, the exception
//! hierarchy, the batch entry point and the release of the interpreter lock are
//! #35, and the dependency that binds to CPython is named there rather than
//! here, because this change is scaffolding and carries no logic.

/// Placeholder, reaching the core the way every consumer of this layer will.
///
/// The path is `indexwerk`, the library the foreign-interface crate builds, and
/// the symbol is one of its exports. It is not `indexwerk_core`, and that is the
/// whole rule of this layer expressed in one line.
pub fn layers() -> u32 {
    indexwerk::indexwerk_layers()
}

#[cfg(test)]
mod tests {
    use super::layers;

    #[test]
    fn the_python_layer_reaches_the_core_through_the_c_interface() {
        assert_eq!(layers(), indexwerk::indexwerk_layers());
    }
}
