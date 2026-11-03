// SPDX-License-Identifier: GPL-3.0-or-later
//! EH3D-COMPONENT-INTELLIGENCE: a technical database of electronic components in which every value says where it came from.
//!
//! The first version is filled by hand or imported from a file; it uses no artificial intelligence and no network. It keeps, for each component, the
//! **observations** of each field (a value, how it was obtained, and the source, page, table, revision and date it was consulted), finds what is wrong with
//! them and refuses to call a record verified while something is pending.
//!
//! Findings carry stable codes `PHOENIX-CINT-NNN` (see [`Code`]). The database is plain JSON and is checked, never repaired: a record is changed by a person.
//!
//! Dates are text supplied by the person who fills the database; this crate never reads the clock, so the same file always gives the same findings.
#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod cli;
pub mod config;
pub mod database;
pub mod error;

pub use database::{Component, Database, Finding, Observation};
pub use error::CintError;

/// The stable code of a finding. Codes are never reused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Code {
    /// Two records have the same manufacturer and part number.
    DuplicateIdentity,
    /// An observation has no source (nothing says where the value came from).
    NoProvenance,
    /// Two observations of one field disagree.
    Conflict,
    /// A record marked `verified` still has fields pending review, a conflict, or an observation that is not verified.
    VerifiedWithPending,
    /// A record has no origin (manual or import) or an import does not say where it came from.
    NoOrigin,
    /// A record has no licence for its data, so it cannot be exported.
    NoLicence,
    /// A record has an empty manufacturer or part number, or a field with no observation.
    BadRecord,
}

impl Code {
    /// The text of the code, such as `PHOENIX-CINT-001`.
    pub fn as_str(self) -> &'static str {
        match self {
            Code::DuplicateIdentity => "PHOENIX-CINT-001",
            Code::NoProvenance => "PHOENIX-CINT-002",
            Code::Conflict => "PHOENIX-CINT-003",
            Code::VerifiedWithPending => "PHOENIX-CINT-004",
            Code::NoOrigin => "PHOENIX-CINT-005",
            Code::NoLicence => "PHOENIX-CINT-006",
            Code::BadRecord => "PHOENIX-CINT-007",
        }
    }
}
