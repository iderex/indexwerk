//! The search terms, and nothing else.
//!
//! This file is excluded from the scan it feeds, because a table of forbidden
//! constructs necessarily contains every forbidden construct. The exclusion is
//! two named files rather than a directory, so everything else in this crate is
//! scanned like any other source.
//!
//! The terms live here rather than in a document on purpose. Widening them is a
//! code change that shows up in a diff and goes through the gate, which is what
//! #17 and #41 both ask for. `docs/invariants.md` is rendered from this table
//! rather than maintained beside it, so a term added here without regenerating
//! that file reds the check.

/// One of the greppable invariants of #41.
///
/// An invariant is the rule a reader is told about. A [`Class`] is the shape of
/// the construct that broke it, and several classes can belong to one
/// invariant: four separate constructs break the headless and unelevated
/// requirement and a reader meeting one of them wants to know which.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Invariant {
    NoUnsafeOutsideTheDeclaredCrate,
    NoFloatingPointInTheCore,
    NoEgressFromAShippedCrate,
    NoPanicPathInALibraryCrate,
    HeadlessAndUnelevated,
    NoPerformanceNumberWithoutItsSource,
}

/// Every invariant, in the order `docs/invariants.md` lists them.
pub const INVARIANTS: &[Invariant] = &[
    Invariant::NoUnsafeOutsideTheDeclaredCrate,
    Invariant::NoFloatingPointInTheCore,
    Invariant::NoEgressFromAShippedCrate,
    Invariant::NoPanicPathInALibraryCrate,
    Invariant::HeadlessAndUnelevated,
    Invariant::NoPerformanceNumberWithoutItsSource,
];

impl Invariant {
    pub fn title(self) -> &'static str {
        match self {
            Invariant::NoUnsafeOutsideTheDeclaredCrate => {
                "No unsafe code outside the one declared crate"
            }
            Invariant::NoFloatingPointInTheCore => "No floating point types in the core crate",
            Invariant::NoEgressFromAShippedCrate => {
                "No socket, name resolution, network client or process spawn in a shipped crate"
            }
            Invariant::NoPanicPathInALibraryCrate => "No panic path in a library crate",
            Invariant::HeadlessAndUnelevated => {
                "No test that binds off loopback, touches a certificate store, installs a \
                 service or asks for elevation"
            }
            Invariant::NoPerformanceNumberWithoutItsSource => {
                "No performance number in tracked documentation without its source nearby"
            }
        }
    }

    /// Where the rule comes from. This is the half of a finding that tells
    /// somebody meeting a red check why the rule exists rather than how to
    /// silence it, so every invariant has one and it names a file or an issue
    /// that can be opened.
    pub fn source(self) -> &'static str {
        match self {
            Invariant::NoUnsafeOutsideTheDeclaredCrate => "docs/adr/0005-layering.md, issue #7",
            Invariant::NoFloatingPointInTheCore => "docs/adr/0007-exact-arithmetic.md, issue #9",
            Invariant::NoEgressFromAShippedCrate => {
                "docs/adr/0008-nothing-leaves-the-host.md, issue #36"
            }
            Invariant::NoPanicPathInALibraryCrate => "issue #41",
            Invariant::HeadlessAndUnelevated => "issue #17",
            Invariant::NoPerformanceNumberWithoutItsSource => "issue #31",
        }
    }

    /// What the invariant reads, in the words a reader needs to know whether
    /// their file is inside it. The machine-readable half is [`Scope`].
    pub fn scope_description(self) -> &'static str {
        self.scope().description()
    }

    pub fn scope(self) -> Scope {
        match self {
            Invariant::NoUnsafeOutsideTheDeclaredCrate => Scope::CratesOtherThanTheForeignInterface,
            Invariant::NoFloatingPointInTheCore => Scope::TheCoreCrate,
            Invariant::NoEgressFromAShippedCrate => Scope::TheShippedCrates,
            Invariant::NoPanicPathInALibraryCrate => Scope::LibrarySourcesOutsideTests,
            Invariant::HeadlessAndUnelevated => Scope::EveryRustSourceUnderCrates,
            Invariant::NoPerformanceNumberWithoutItsSource => Scope::TrackedDocumentation,
        }
    }
}

/// Which files an invariant reads.
///
/// The scope is data rather than a condition written at each call site, because
/// an invariant whose reach is a branch buried in the walk is an invariant
/// nobody can state. Each variant is resolved by one function,
/// [`Scope::covers`], and that function is what the scope tests exercise.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Scope {
    /// Every `.rs` file under `crates/`, shipped or not.
    EveryRustSourceUnderCrates,
    /// Every `.rs` file under `crates/` except those of the crate that is the
    /// declared exception to `#![forbid(unsafe_code)]`.
    CratesOtherThanTheForeignInterface,
    /// The core crate only.
    TheCoreCrate,
    /// The three crates that reach a consumer: the core, the C interface and
    /// the Python layer. The checks crate ships to nobody and is outside this.
    TheShippedCrates,
    /// `src/` of every crate in the workspace, with `#[cfg(test)]` regions
    /// skipped.
    LibrarySourcesOutsideTests,
    /// Markdown tracked as documentation: `README.md` and everything under
    /// `docs/`.
    TrackedDocumentation,
}

impl Scope {
    pub fn description(self) -> &'static str {
        match self {
            Scope::EveryRustSourceUnderCrates => "every Rust source under crates/",
            Scope::CratesOtherThanTheForeignInterface => {
                "every Rust source under crates/ except crates/indexwerk-ffi/"
            }
            Scope::TheCoreCrate => "crates/indexwerk-core/",
            Scope::TheShippedCrates => {
                "crates/indexwerk-core/, crates/indexwerk-ffi/ and crates/indexwerk-python/"
            }
            Scope::LibrarySourcesOutsideTests => {
                "src/ of every crate, outside #[cfg(test)] regions"
            }
            Scope::TrackedDocumentation => "README.md and docs/",
        }
    }

    /// Whether a path, relative to the workspace root and written with forward
    /// slashes, is inside this scope.
    pub fn covers(self, relative_path: &str) -> bool {
        let rust = relative_path.starts_with("crates/") && relative_path.ends_with(".rs");
        match self {
            Scope::EveryRustSourceUnderCrates => rust,
            Scope::CratesOtherThanTheForeignInterface => {
                rust && !relative_path.starts_with(FOREIGN_INTERFACE_CRATE)
            }
            Scope::TheCoreCrate => rust && relative_path.starts_with(CORE_CRATE),
            Scope::TheShippedCrates => {
                rust && SHIPPED_CRATES
                    .iter()
                    .any(|prefix| relative_path.starts_with(prefix))
            }
            Scope::LibrarySourcesOutsideTests => {
                rust && relative_path.split('/').any(|segment| segment == "src")
            }
            Scope::TrackedDocumentation => {
                relative_path == "README.md"
                    || (relative_path.starts_with("docs/") && relative_path.ends_with(".md"))
            }
        }
    }
}

/// What a term is about, and what a reader meeting it should be told.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Class {
    /// An unsafe block, function, implementation or attribute.
    UnsafeConstruct,
    /// A crate that should refuse unsafe code at compile time and no longer
    /// does. This is the half that catches deleting the attribute rather than
    /// writing an unsafe block, which is the gap #7 names.
    MissingCompileTimeRefusal,
    /// A floating point type in the core.
    FloatingPointType,
    /// A socket, a name resolution, a network client or a process spawn.
    Egress,
    /// An unwrap, an expect, or a macro that aborts.
    PanicPath,
    /// A socket bound to something other than loopback. On Windows this raises
    /// a firewall consent dialog that only an administrator can answer, and the
    /// answer covers one executable path rather than the project, so every new
    /// build directory asks again.
    OffLoopbackBind,
    /// Reading or writing a certificate store.
    CertificateStore,
    /// Installing or registering a service or a scheduled task.
    ServiceInstall,
    /// Asking for administrator rights.
    Elevation,
    /// A time figure in documentation with neither a command nor a label saying
    /// where it came from.
    UnsourcedPerformanceNumber,
    /// A file the walk reached and could not read. Not a construct, and it is a
    /// finding anyway: a file that cannot be read is not a file that was found
    /// clean.
    UnreadableFile,
}

impl Class {
    pub fn name(self) -> &'static str {
        match self {
            Class::UnsafeConstruct => "unsafe code outside the declared crate",
            Class::MissingCompileTimeRefusal => "crate root without #![forbid(unsafe_code)]",
            Class::FloatingPointType => "floating point type in the core",
            Class::Egress => "socket, name resolution, network client or process spawn",
            Class::PanicPath => "panic path in a library crate",
            Class::OffLoopbackBind => "socket bound off loopback",
            Class::CertificateStore => "certificate store access",
            Class::ServiceInstall => "service or scheduled task installation",
            Class::Elevation => "process elevation",
            Class::UnsourcedPerformanceNumber => "performance number with no source nearby",
            Class::UnreadableFile => "file could not be read",
        }
    }

    pub fn invariant(self) -> Invariant {
        match self {
            Class::UnsafeConstruct | Class::MissingCompileTimeRefusal => {
                Invariant::NoUnsafeOutsideTheDeclaredCrate
            }
            Class::FloatingPointType => Invariant::NoFloatingPointInTheCore,
            Class::Egress => Invariant::NoEgressFromAShippedCrate,
            Class::PanicPath => Invariant::NoPanicPathInALibraryCrate,
            Class::OffLoopbackBind
            | Class::CertificateStore
            | Class::ServiceInstall
            | Class::Elevation
            | Class::UnreadableFile => Invariant::HeadlessAndUnelevated,
            Class::UnsourcedPerformanceNumber => Invariant::NoPerformanceNumberWithoutItsSource,
        }
    }
}

/// How a needle is matched.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Match {
    /// Anywhere in the line. Every needle matched this way is a spelling
    /// somebody wrote deliberately and does not occur inside a longer word.
    Substring,
    /// Bounded on both sides by something that is not a letter, a digit or an
    /// underscore, so that `f64` does not fire on `buf64`.
    Token,
}

/// One forbidden construct.
pub struct Term {
    /// The literal to look for, matched case sensitively.
    pub needle: &'static str,
    pub class: Class,
    pub matching: Match,
}

/// The crate that is the declared exception to `#![forbid(unsafe_code)]`.
pub const FOREIGN_INTERFACE_CRATE: &str = "crates/indexwerk-ffi/";

/// The core.
pub const CORE_CRATE: &str = "crates/indexwerk-core/";

/// The crates that reach a consumer.
pub const SHIPPED_CRATES: &[&str] = &[
    "crates/indexwerk-core/",
    "crates/indexwerk-ffi/",
    "crates/indexwerk-python/",
];

/// Crate roots that must carry the compile-time refusal, relative to the
/// workspace root. `crates/indexwerk-ffi/src/lib.rs` is deliberately absent and
/// that absence is the exception `docs/adr/0005-layering.md` grants.
pub const ROOTS_THAT_MUST_FORBID_UNSAFE: &[&str] = &[
    "crates/indexwerk-checks/src/lib.rs",
    "crates/indexwerk-core/src/lib.rs",
    "crates/indexwerk-python/src/lib.rs",
];

/// The attribute those roots must carry.
pub const COMPILE_TIME_REFUSAL: &str = "#![forbid(unsafe_code)]";

/// A line carrying an off-loopback term is allowed if it also carries one of
/// these. A bind to loopback is exactly what the rule permits, so refusing it
/// would refuse the compliant form along with the violating one.
pub const LOOPBACK_ESCAPES: &[&str] = &[
    "127.0.0.1",
    "::1",
    "localhost",
    "Ipv4Addr::LOCALHOST",
    "Ipv6Addr::LOCALHOST",
];

/// The comment that admits one panic path.
///
/// #41 permits a small named list of places where a violated internal invariant
/// genuinely should abort, each carrying a comment saying why. This is that
/// comment, and it is a literal rather than a shape so that the list of such
/// places is one grep away. It is honoured on the line it is written on and on
/// the line below it, so it can sit above the statement it admits.
pub const ABORT_IS_CORRECT_MARKER: &str = "// aborts on a violated internal invariant:";

/// The two files excluded from the source scan, relative to the workspace root,
/// with forward slashes. Both contain the literals above as data. Every other
/// file in the tree is scanned.
pub const EXCLUDED: &[&str] = &[
    "crates/indexwerk-checks/src/terms.rs",
    "crates/indexwerk-checks/tests/bites.rs",
];

/// Units a performance figure is written in, ASCII spellings only.
///
/// The check is deliberately crude, which #41 says in as many words: a crude
/// check that fires on a real defect is worth more than an exact one nobody
/// writes. The bound is disclosed rather than hidden. A figure written `2.4 s`
/// is caught, one written `2400 milliseconds` is not, and one written with a
/// non-ASCII micro sign is not.
pub const TIME_UNITS: &[&str] = &["ms", "us", "ns", "s"];

/// A performance figure is allowed if one of these appears within
/// [`SOURCE_DISTANCE`] lines of it, in either direction.
///
/// A command block is the measured case: the number carries the command that
/// produced it. The words are the published case: the number is somebody
/// else's and is labelled that way, which is the distinction #31 fixes.
pub const SOURCE_WORDS: &[&str] = &["published", "Published", "quoted", "not measured"];

/// How far a source may sit from the figure it accounts for.
pub const SOURCE_DISTANCE: usize = 10;

pub const TERMS: &[Term] = &[
    // Unsafe, in the spellings the compiler accepts. `unsafe_code` inside the
    // forbid attribute is not one of them and is not matched, so the attribute
    // that enforces the rule does not trip the check that enforces the rule.
    Term {
        needle: "unsafe {",
        class: Class::UnsafeConstruct,
        matching: Match::Substring,
    },
    Term {
        needle: "unsafe fn",
        class: Class::UnsafeConstruct,
        matching: Match::Substring,
    },
    Term {
        needle: "unsafe impl",
        class: Class::UnsafeConstruct,
        matching: Match::Substring,
    },
    Term {
        needle: "unsafe trait",
        class: Class::UnsafeConstruct,
        matching: Match::Substring,
    },
    Term {
        needle: "unsafe extern",
        class: Class::UnsafeConstruct,
        matching: Match::Substring,
    },
    Term {
        needle: "#[unsafe(",
        class: Class::UnsafeConstruct,
        matching: Match::Substring,
    },
    // Floating point, as tokens, so that a name ending in one of them is not a
    // finding.
    Term {
        needle: "f32",
        class: Class::FloatingPointType,
        matching: Match::Token,
    },
    Term {
        needle: "f64",
        class: Class::FloatingPointType,
        matching: Match::Token,
    },
    // Anything that could carry a byte off this host, or start something that
    // would. The core performs no input or output at all, and the two layers
    // above it cross a boundary rather than a network.
    Term {
        needle: "std::net",
        class: Class::Egress,
        matching: Match::Substring,
    },
    Term {
        needle: "TcpListener",
        class: Class::Egress,
        matching: Match::Substring,
    },
    Term {
        needle: "TcpStream",
        class: Class::Egress,
        matching: Match::Substring,
    },
    Term {
        needle: "UdpSocket",
        class: Class::Egress,
        matching: Match::Substring,
    },
    Term {
        needle: "ToSocketAddrs",
        class: Class::Egress,
        matching: Match::Substring,
    },
    Term {
        needle: "to_socket_addrs",
        class: Class::Egress,
        matching: Match::Substring,
    },
    Term {
        needle: "lookup_host",
        class: Class::Egress,
        matching: Match::Substring,
    },
    Term {
        needle: "reqwest",
        class: Class::Egress,
        matching: Match::Substring,
    },
    Term {
        needle: "ureq",
        class: Class::Egress,
        matching: Match::Substring,
    },
    Term {
        needle: "hyper::",
        class: Class::Egress,
        matching: Match::Substring,
    },
    Term {
        needle: "Command::new",
        class: Class::Egress,
        matching: Match::Substring,
    },
    Term {
        needle: "std::process::Command",
        class: Class::Egress,
        matching: Match::Substring,
    },
    // Panic paths. `expect_err` and friends are test vocabulary and live in
    // regions this invariant skips, so the open parenthesis is enough to keep
    // the needle off a name that merely starts the same way.
    Term {
        needle: ".unwrap()",
        class: Class::PanicPath,
        matching: Match::Substring,
    },
    Term {
        needle: ".expect(",
        class: Class::PanicPath,
        matching: Match::Substring,
    },
    Term {
        needle: "panic!(",
        class: Class::PanicPath,
        matching: Match::Substring,
    },
    Term {
        needle: "todo!(",
        class: Class::PanicPath,
        matching: Match::Substring,
    },
    Term {
        needle: "unimplemented!(",
        class: Class::PanicPath,
        matching: Match::Substring,
    },
    Term {
        needle: "unreachable!(",
        class: Class::PanicPath,
        matching: Match::Substring,
    },
    // A socket bound to a wildcard or a routable address. The escapes above
    // keep a loopback bind legal.
    Term {
        needle: "0.0.0.0",
        class: Class::OffLoopbackBind,
        matching: Match::Substring,
    },
    Term {
        needle: "[::]",
        class: Class::OffLoopbackBind,
        matching: Match::Substring,
    },
    Term {
        needle: "Ipv4Addr::UNSPECIFIED",
        class: Class::OffLoopbackBind,
        matching: Match::Substring,
    },
    Term {
        needle: "Ipv6Addr::UNSPECIFIED",
        class: Class::OffLoopbackBind,
        matching: Match::Substring,
    },
    // Certificate stores, on each platform that has one.
    Term {
        needle: "CertOpenStore",
        class: Class::CertificateStore,
        matching: Match::Substring,
    },
    Term {
        needle: "CertAddCertificateContextToStore",
        class: Class::CertificateStore,
        matching: Match::Substring,
    },
    Term {
        needle: "X509Store",
        class: Class::CertificateStore,
        matching: Match::Substring,
    },
    Term {
        needle: "dev-certs",
        class: Class::CertificateStore,
        matching: Match::Substring,
    },
    Term {
        needle: "certutil",
        class: Class::CertificateStore,
        matching: Match::Substring,
    },
    Term {
        needle: "add-trusted-cert",
        class: Class::CertificateStore,
        matching: Match::Substring,
    },
    Term {
        needle: "update-ca-certificates",
        class: Class::CertificateStore,
        matching: Match::Substring,
    },
    // Services and scheduled tasks.
    Term {
        needle: "sc.exe",
        class: Class::ServiceInstall,
        matching: Match::Substring,
    },
    Term {
        needle: "New-Service",
        class: Class::ServiceInstall,
        matching: Match::Substring,
    },
    Term {
        needle: "CreateServiceW",
        class: Class::ServiceInstall,
        matching: Match::Substring,
    },
    Term {
        needle: "CreateServiceA",
        class: Class::ServiceInstall,
        matching: Match::Substring,
    },
    Term {
        needle: "systemctl enable",
        class: Class::ServiceInstall,
        matching: Match::Substring,
    },
    Term {
        needle: "launchctl load",
        class: Class::ServiceInstall,
        matching: Match::Substring,
    },
    Term {
        needle: "schtasks",
        class: Class::ServiceInstall,
        matching: Match::Substring,
    },
    // Elevation, in the spellings that actually appear.
    Term {
        needle: "runas",
        class: Class::Elevation,
        matching: Match::Substring,
    },
    Term {
        needle: "RunAs",
        class: Class::Elevation,
        matching: Match::Substring,
    },
    Term {
        needle: "requireAdministrator",
        class: Class::Elevation,
        matching: Match::Substring,
    },
    Term {
        needle: "AdjustTokenPrivileges",
        class: Class::Elevation,
        matching: Match::Substring,
    },
    Term {
        needle: "ShellExecuteW",
        class: Class::Elevation,
        matching: Match::Substring,
    },
    Term {
        needle: "sudo ",
        class: Class::Elevation,
        matching: Match::Substring,
    },
    Term {
        needle: "gsudo",
        class: Class::Elevation,
        matching: Match::Substring,
    },
];
