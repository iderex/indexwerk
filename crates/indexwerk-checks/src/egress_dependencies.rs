//! Nothing in the locked tree brings a route off this host with it (#36).
//!
//! `docs/adr/0008-nothing-leaves-the-host.md` says the library opens no socket,
//! resolves no name and sends nothing anywhere. The source-level half of that is
//! the greppable invariant in [`crate::terms`], which reads the code somebody
//! wrote here. This is the other half, and it is the one that catches the real
//! risk: not that somebody writes a socket call, but that a convenient crate
//! arrives with one attached three levels down, where nobody reading this
//! repository's own sources would ever see it.
//!
//! It reads `Cargo.lock`, which is the whole transitive set rather than the
//! direct edges, so a stack pulled in by something pulled in by something is in
//! what is judged. The walk itself is [`crate::dependencies::locked_in`], shared
//! with the register of #38 rather than written a second time, because two walks
//! are how two lists end up disagreeing about what is in the tree.
//!
//! The allow list is empty and adding to it needs an issue. That is written here
//! because a list somebody may add to has to say what adding costs, next to the
//! list rather than in a document somebody would have to already know about.
//!
//! What this cannot do, in three parts, each of which is real.
//!
//! It is a list of names. A crate whose name is not on it walks through, exactly
//! as a construct in an unnamed spelling walks through the source scan. Widening
//! the list is a change to this file that shows up in a diff and goes through
//! the gate, which is the repair rather than an apology.
//!
//! A lock file records crates and versions and never features. `tokio` without
//! its networking features opens nothing, and this refuses it anyway, because
//! the file being read cannot tell the two apart. Refusing the compiling case
//! along with the violating one is the direction this errs in deliberately: the
//! cost is an issue arguing for an entry on the allow list, and the cost the
//! other way is a guarantee that is not one.
//!
//! A crate can carry a route out under a name that says nothing about it. That
//! is the residual this check does not reach and the behavioural test of #36 is
//! what would, by running with networking unavailable and passing. That test
//! does not exist.

use crate::dependencies::locked_in;

/// What a refused crate brings with it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Brings {
    /// Sockets, name resolution, or a client speaking a protocol over them.
    ANetworkStack,
    /// A client whose purpose is reporting usage somewhere.
    ATelemetryClient,
    /// A reporter whose purpose is sending a crash somewhere. The decision
    /// record names this case in as many words: a crash report carrying the
    /// expression that crashed exfiltrates the operator's unpublished work.
    ACrashReporter,
}

impl Brings {
    pub fn description(self) -> &'static str {
        match self {
            Brings::ANetworkStack => "a network stack",
            Brings::ATelemetryClient => "a telemetry client",
            Brings::ACrashReporter => "a crash reporter",
        }
    }
}

/// One crate that may not be in the locked tree, and what it brings.
pub struct Forbidden {
    pub name: &'static str,
    pub brings: Brings,
}

/// The names refused, whether they are a direct dependency or arrive three
/// levels down.
///
/// Each is here because reaching a machine that is not this one is what it is
/// for, or because it exists only to secure such a reach. The transport crates
/// are named as well as the clients: refusing `reqwest` and admitting `hyper`
/// would refuse the convenient spelling and take the same route by a longer
/// name.
pub const FORBIDDEN: &[Forbidden] = &[
    // Sockets and the runtimes whose reason to exist is driving them.
    Forbidden {
        name: "socket2",
        brings: Brings::ANetworkStack,
    },
    Forbidden {
        name: "mio",
        brings: Brings::ANetworkStack,
    },
    Forbidden {
        name: "tokio",
        brings: Brings::ANetworkStack,
    },
    Forbidden {
        name: "async-std",
        brings: Brings::ANetworkStack,
    },
    Forbidden {
        name: "smol",
        brings: Brings::ANetworkStack,
    },
    Forbidden {
        name: "polling",
        brings: Brings::ANetworkStack,
    },
    // Name resolution.
    Forbidden {
        name: "hickory-resolver",
        brings: Brings::ANetworkStack,
    },
    Forbidden {
        name: "trust-dns-resolver",
        brings: Brings::ANetworkStack,
    },
    Forbidden {
        name: "dns-lookup",
        brings: Brings::ANetworkStack,
    },
    // Clients and the transports underneath them.
    Forbidden {
        name: "reqwest",
        brings: Brings::ANetworkStack,
    },
    Forbidden {
        name: "hyper",
        brings: Brings::ANetworkStack,
    },
    Forbidden {
        name: "h2",
        brings: Brings::ANetworkStack,
    },
    Forbidden {
        name: "h3",
        brings: Brings::ANetworkStack,
    },
    Forbidden {
        name: "ureq",
        brings: Brings::ANetworkStack,
    },
    Forbidden {
        name: "attohttpc",
        brings: Brings::ANetworkStack,
    },
    Forbidden {
        name: "isahc",
        brings: Brings::ANetworkStack,
    },
    Forbidden {
        name: "surf",
        brings: Brings::ANetworkStack,
    },
    Forbidden {
        name: "curl",
        brings: Brings::ANetworkStack,
    },
    Forbidden {
        name: "curl-sys",
        brings: Brings::ANetworkStack,
    },
    Forbidden {
        name: "quinn",
        brings: Brings::ANetworkStack,
    },
    Forbidden {
        name: "tungstenite",
        brings: Brings::ANetworkStack,
    },
    Forbidden {
        name: "tokio-tungstenite",
        brings: Brings::ANetworkStack,
    },
    // Transport security. Nothing here needs a secure channel, because nothing
    // here opens a channel, so one of these in the tree is a channel somebody
    // else opened.
    Forbidden {
        name: "rustls",
        brings: Brings::ANetworkStack,
    },
    Forbidden {
        name: "native-tls",
        brings: Brings::ANetworkStack,
    },
    Forbidden {
        name: "openssl",
        brings: Brings::ANetworkStack,
    },
    // Reporting usage.
    Forbidden {
        name: "opentelemetry",
        brings: Brings::ATelemetryClient,
    },
    Forbidden {
        name: "opentelemetry-otlp",
        brings: Brings::ATelemetryClient,
    },
    Forbidden {
        name: "tracing-opentelemetry",
        brings: Brings::ATelemetryClient,
    },
    Forbidden {
        name: "metrics-exporter-prometheus",
        brings: Brings::ATelemetryClient,
    },
    Forbidden {
        name: "statsd",
        brings: Brings::ATelemetryClient,
    },
    // Reporting a crash.
    Forbidden {
        name: "sentry",
        brings: Brings::ACrashReporter,
    },
    Forbidden {
        name: "sentry-core",
        brings: Brings::ACrashReporter,
    },
    Forbidden {
        name: "minidumper",
        brings: Brings::ACrashReporter,
    },
    Forbidden {
        name: "crash-handler",
        brings: Brings::ACrashReporter,
    },
];

/// Crates admitted although they are on the list above.
///
/// It is empty at the first release and it is empty now. Adding an entry needs
/// an issue arguing why a route off this host is acceptable in a library whose
/// decision record says there is none, and the entry names that issue. An entry
/// added without one is the shape this whole check exists against, so the cost
/// is written next to the list rather than somewhere a reader would have to
/// already know about.
pub const ALLOWED: &[&str] = &[];

/// One locked crate that may not be there.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Wrong {
    pub name: String,
    pub version: String,
    pub brings: Brings,
}

impl Wrong {
    pub fn message(&self) -> String {
        format!(
            "{:?} {} is in the locked tree and it brings {} with it. \
             docs/adr/0008-nothing-leaves-the-host.md says the library has no route off this \
             host, and a route arriving through a dependency is still a route. Issue #36",
            self.name,
            self.version,
            self.brings.description()
        )
    }
}

/// Judge the text of a lock file.
///
/// Split from the file reading so a fixture can be judged without existing on
/// disk, which is how the refusal below is proved.
pub fn judge(lock: &str) -> Vec<Wrong> {
    let mut wrongs = Vec::new();
    for package in locked_in(lock) {
        if ALLOWED.contains(&package.name.as_str()) {
            continue;
        }
        if let Some(entry) = FORBIDDEN
            .iter()
            .find(|entry| entry.name == package.name.as_str())
        {
            wrongs.push(Wrong {
                name: package.name,
                version: package.version,
                brings: entry.brings,
            });
        }
    }
    wrongs
}

/// Judge this tree's lock file. Nothing where the file cannot be read: that
/// case is already a refusal of the register in [`crate::dependencies`], and
/// reporting it twice would say one absence is two problems.
pub fn judge_the_tree() -> Vec<Wrong> {
    let lock = std::fs::read_to_string(crate::workspace_root().join("Cargo.lock"));
    match lock {
        Ok(text) => judge(&text),
        Err(_) => Vec::new(),
    }
}
