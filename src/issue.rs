// SPDX-License-Identifier: GPL-3.0-or-later
//! The refusals of the import and of the writing of a database: each one has a stable code `EH3D-CINT-NNN` that is never reused.

use std::fmt;

/// The stable code of an import or storage refusal. Codes are never reused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ImportCode {
    /// A size limit was exceeded (bytes of the file, components, observations of a field).
    TooLarge,
    /// The file has more data rows than the limit.
    TooManyRows,
    /// A line has more columns than the limit.
    TooManyColumns,
    /// A field is longer than the limit.
    FieldTooLong,
    /// The file is not valid UTF-8.
    BadEncoding,
    /// A quotation mark is used in a way the format does not allow, or a bare carriage return.
    BadQuote,
    /// The header is missing, lacks a required column or repeats a column.
    BadHeader,
    /// The header has a column this version does not know.
    UnknownColumn,
    /// A line has a different number of fields than the header.
    RowWidth,
    /// A required value (reference, manufacturer or mpn) is empty.
    EmptyRequired,
    /// Two lines use the same reference.
    DuplicateReference,
    /// The output file exists and no explicit order to replace it was given.
    OutputExists,
    /// The output file could not be written.
    WriteFailed,
}

impl ImportCode {
    /// The text of the code, such as `EH3D-CINT-101`.
    pub fn as_str(self) -> &'static str {
        match self {
            ImportCode::TooLarge => "EH3D-CINT-101",
            ImportCode::TooManyRows => "EH3D-CINT-102",
            ImportCode::TooManyColumns => "EH3D-CINT-103",
            ImportCode::FieldTooLong => "EH3D-CINT-104",
            ImportCode::BadEncoding => "EH3D-CINT-105",
            ImportCode::BadQuote => "EH3D-CINT-106",
            ImportCode::BadHeader => "EH3D-CINT-107",
            ImportCode::UnknownColumn => "EH3D-CINT-108",
            ImportCode::RowWidth => "EH3D-CINT-109",
            ImportCode::EmptyRequired => "EH3D-CINT-110",
            ImportCode::DuplicateReference => "EH3D-CINT-111",
            ImportCode::OutputExists => "EH3D-CINT-112",
            ImportCode::WriteFailed => "EH3D-CINT-113",
        }
    }
}

/// One refusal: the code, the 1-based line of the file it is about (0 when it is about the whole file) and what is wrong.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Issue {
    /// The code.
    pub code: ImportCode,
    /// The line of the file, or 0.
    pub line: usize,
    /// What is wrong, in English.
    pub message: String,
}

impl Issue {
    /// A new refusal.
    pub fn new(code: ImportCode, line: usize, message: impl Into<String>) -> Issue {
        Issue {
            code,
            line,
            message: message.into(),
        }
    }
}

impl fmt::Display for Issue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.line == 0 {
            write!(f, "{}: {}", self.code.as_str(), self.message)
        } else {
            write!(
                f,
                "{} line {}: {}",
                self.code.as_str(),
                self.line,
                self.message
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_are_stable_and_distinct() {
        let all = [
            ImportCode::TooLarge,
            ImportCode::TooManyRows,
            ImportCode::TooManyColumns,
            ImportCode::FieldTooLong,
            ImportCode::BadEncoding,
            ImportCode::BadQuote,
            ImportCode::BadHeader,
            ImportCode::UnknownColumn,
            ImportCode::RowWidth,
            ImportCode::EmptyRequired,
            ImportCode::DuplicateReference,
            ImportCode::OutputExists,
            ImportCode::WriteFailed,
        ];
        for (i, code) in all.iter().enumerate() {
            assert_eq!(code.as_str(), format!("EH3D-CINT-{}", 101 + i));
        }
    }

    #[test]
    fn the_text_names_the_line_only_when_there_is_one() {
        assert_eq!(
            Issue::new(ImportCode::RowWidth, 4, "x").to_string(),
            "EH3D-CINT-109 line 4: x"
        );
        assert_eq!(
            Issue::new(ImportCode::TooLarge, 0, "y").to_string(),
            "EH3D-CINT-101: y"
        );
    }
}
