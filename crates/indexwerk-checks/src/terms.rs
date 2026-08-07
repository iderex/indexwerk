//! The search terms, and nothing else.
//!
//! This file is excluded from the scan it feeds, because a table of forbidden
//! constructs necessarily contains every forbidden construct. The exclusion is
//! two named files rather than a directory, so everything else in this crate is
//! scanned like any other source.
//!
//! The terms live here rather than in a document on purpose. Widening them is a
//! code change that shows up in a diff and goes through the gate, which is what
//! #17 asks for.

/// What a term is about, and what a reader meeting it should be told.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Class {
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
}

impl Class {
    pub fn name(self) -> &'static str {
        match self {
            Class::OffLoopbackBind => "socket bound off loopback",
            Class::CertificateStore => "certificate store access",
            Class::ServiceInstall => "service or scheduled task installation",
            Class::Elevation => "process elevation",
        }
    }
}

/// One forbidden construct.
pub struct Term {
    /// The literal to look for. Matching is on the literal, case sensitively,
    /// because every one of these is a spelling somebody wrote deliberately.
    pub needle: &'static str,
    pub class: Class,
}

/// A line carrying an off-loopback term is allowed if it also carries one of
/// these. A bind to loopback is exactly what the rule permits, so refusing it
/// would refuse the compliant form along with the violating one.
pub const LOOPBACK_ESCAPES: &[&str] = &["127.0.0.1", "::1", "localhost", "Ipv4Addr::LOCALHOST", "Ipv6Addr::LOCALHOST"];

/// The two files excluded from the scan, relative to the workspace root, with
/// forward slashes. Both contain the literals above as data. Every other file
/// in the tree is scanned.
pub const EXCLUDED: &[&str] = &[
    "crates/indexwerk-checks/src/terms.rs",
    "crates/indexwerk-checks/tests/bites.rs",
];

pub const TERMS: &[Term] = &[
    // A socket bound to a wildcard or a routable address. The escapes above
    // keep a loopback bind legal.
    Term { needle: "0.0.0.0", class: Class::OffLoopbackBind },
    Term { needle: "[::]", class: Class::OffLoopbackBind },
    Term { needle: "Ipv4Addr::UNSPECIFIED", class: Class::OffLoopbackBind },
    Term { needle: "Ipv6Addr::UNSPECIFIED", class: Class::OffLoopbackBind },
    // Certificate stores, on each platform that has one.
    Term { needle: "CertOpenStore", class: Class::CertificateStore },
    Term { needle: "CertAddCertificateContextToStore", class: Class::CertificateStore },
    Term { needle: "X509Store", class: Class::CertificateStore },
    Term { needle: "dev-certs", class: Class::CertificateStore },
    Term { needle: "certutil", class: Class::CertificateStore },
    Term { needle: "add-trusted-cert", class: Class::CertificateStore },
    Term { needle: "update-ca-certificates", class: Class::CertificateStore },
    // Services and scheduled tasks.
    Term { needle: "sc.exe", class: Class::ServiceInstall },
    Term { needle: "New-Service", class: Class::ServiceInstall },
    Term { needle: "CreateServiceW", class: Class::ServiceInstall },
    Term { needle: "CreateServiceA", class: Class::ServiceInstall },
    Term { needle: "systemctl enable", class: Class::ServiceInstall },
    Term { needle: "launchctl load", class: Class::ServiceInstall },
    Term { needle: "schtasks", class: Class::ServiceInstall },
    // Elevation, in the spellings that actually appear.
    Term { needle: "runas", class: Class::Elevation },
    Term { needle: "RunAs", class: Class::Elevation },
    Term { needle: "requireAdministrator", class: Class::Elevation },
    Term { needle: "AdjustTokenPrivileges", class: Class::Elevation },
    Term { needle: "ShellExecuteW", class: Class::Elevation },
    Term { needle: "sudo ", class: Class::Elevation },
    Term { needle: "gsudo", class: Class::Elevation },
];
