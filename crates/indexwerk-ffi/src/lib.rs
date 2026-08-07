// This crate is the declared exception to `#![forbid(unsafe_code)]`, and it is
// the only one. `docs/adr/0005-layering.md` is where that is decided and why:
// putting every unsafe line in one small crate makes the audit surface a fixed,
// named set of files instead of a property of the whole tree. Removing the
// attribute from any other crate is refused by a check in M8, so this exception
// stays one file deep.
//
// The interface itself, its ownership, nullability and thread-safety rules, and
// the generated header, are #34. Nothing of that is here yet.

// The MSVC linker prints an informational line on stdout whenever it produces
// an import library for a cdylib, in the host's language, and the
// `linker_messages` lint reports that line as a warning. It says nothing about
// this code and no change to this code removes it. It is not allowed here,
// because `linker_messages` does not exist on every toolchain this workspace
// has to build on and an allow of a lint a compiler does not know is itself a
// warning. Where the exception is needed it is granted per job, on the leg that
// meets it, and the reason is written there.

//! The C interface to the canonicalisation core.
//!
//! Symbols carry the `indexwerk_` prefix so that linking alongside another
//! tensor library does not collide.

/// Placeholder export, so that the scaffolding has one symbol crossing the
/// boundary and a test that calls it.
///
/// It takes no pointer and owns nothing, so it needs no ownership rule yet.
/// The functions that do arrive in #34 with those rules stated in the header.
#[unsafe(no_mangle)]
pub extern "C" fn indexwerk_layers() -> u32 {
    indexwerk_core::layers()
}

#[cfg(test)]
mod tests {
    use super::indexwerk_layers;

    #[test]
    fn the_exported_symbol_returns_what_the_core_returns() {
        assert_eq!(indexwerk_layers(), indexwerk_core::layers());
    }
}
