// SPDX-License-Identifier: GPL-3.0-or-later
//! The reader of the CSV files: strict, bounded and free of any other module of the crate (see `docs/CSV_FORMAT.md`).
//!
//! The rules: UTF-8 with or without a byte order mark; lines end in LF or CRLF; fields are separated by one configurable character; a field that
//! starts with a quotation mark ends at the next lone one (`""` stands for a quotation mark, and the field may hold separators and line breaks);
//! a quotation mark anywhere else is refused; blank lines are skipped; the first line is the header and every other line must have as many fields
//! as the header. Nothing is trimmed here except the names of the header (trimmed and in lower case).

use crate::issue::{ImportCode, Issue};

/// The bounds of the reader.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CsvLimits {
    /// The most bytes the file may have.
    pub max_bytes: usize,
    /// The most data rows (the header does not count).
    pub max_rows: usize,
    /// The most columns of a line.
    pub max_columns: usize,
    /// The most bytes of one field.
    pub max_field_bytes: usize,
}

/// One data row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CsvRow {
    /// The 1-based line of the file where the row starts.
    pub line: usize,
    /// The fields, as written (quotes removed, nothing trimmed).
    pub fields: Vec<String>,
}

/// A parsed file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CsvTable {
    /// The line of the header.
    pub header_line: usize,
    /// The names of the header, trimmed and in lower case.
    pub header: Vec<String>,
    /// The data rows, all as wide as the header.
    pub rows: Vec<CsvRow>,
}

fn field_too_long(line: usize, limits: &CsvLimits) -> Issue {
    Issue::new(
        ImportCode::FieldTooLong,
        line,
        format!("a field is longer than {} bytes", limits.max_field_bytes),
    )
}

/// Parse a CSV file. The first problem stops the reading.
pub fn parse(bytes: &[u8], separator: char, limits: &CsvLimits) -> Result<CsvTable, Issue> {
    if bytes.len() > limits.max_bytes {
        return Err(Issue::new(
            ImportCode::TooLarge,
            0,
            format!("the file is larger than {} bytes", limits.max_bytes),
        ));
    }
    let bytes = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF][..]).unwrap_or(bytes);
    let text = std::str::from_utf8(bytes).map_err(|e| {
        let line = 1 + bytes[..e.valid_up_to()]
            .iter()
            .filter(|b| **b == b'\n')
            .count();
        Issue::new(ImportCode::BadEncoding, line, "the file is not valid UTF-8")
    })?;
    let mut chars = text.chars().peekable();
    let mut line = 1usize;
    let mut header: Option<Vec<String>> = None;
    let mut header_line = 1usize;
    let mut rows: Vec<CsvRow> = Vec::new();
    while chars.peek().is_some() {
        let start = line;
        let mut fields: Vec<String> = Vec::new();
        let mut first_quoted = false;
        loop {
            let mut field = String::new();
            let quoted = chars.peek() == Some(&'"');
            if fields.is_empty() {
                first_quoted = quoted;
            }
            if quoted {
                let open = line;
                chars.next();
                loop {
                    match chars.next() {
                        None => {
                            return Err(Issue::new(
                                ImportCode::BadQuote,
                                open,
                                "a quotation mark is opened and never closed",
                            ))
                        }
                        Some('"') => {
                            if chars.peek() == Some(&'"') {
                                chars.next();
                                field.push('"');
                            } else {
                                break;
                            }
                        }
                        Some(c) => {
                            if c == '\n' {
                                line += 1;
                            }
                            field.push(c);
                        }
                    }
                    if field.len() > limits.max_field_bytes {
                        return Err(field_too_long(open, limits));
                    }
                }
                match chars.peek() {
                    None | Some('\n') | Some('\r') => {}
                    Some(c) if *c == separator => {}
                    Some(_) => {
                        return Err(Issue::new(
                            ImportCode::BadQuote,
                            line,
                            "text follows the closing quotation mark of a field",
                        ))
                    }
                }
            } else {
                while let Some(&c) = chars.peek() {
                    if c == separator || c == '\n' || c == '\r' {
                        break;
                    }
                    if c == '"' {
                        return Err(Issue::new(
                            ImportCode::BadQuote,
                            line,
                            "a quotation mark inside a field that does not start with one",
                        ));
                    }
                    field.push(c);
                    chars.next();
                    if field.len() > limits.max_field_bytes {
                        return Err(field_too_long(line, limits));
                    }
                }
            }
            fields.push(field);
            if fields.len() > limits.max_columns {
                return Err(Issue::new(
                    ImportCode::TooManyColumns,
                    start,
                    format!("a line has more than {} columns", limits.max_columns),
                ));
            }
            match chars.next() {
                Some(c) if c == separator => continue,
                Some('\r') => {
                    if chars.next() != Some('\n') {
                        return Err(Issue::new(
                            ImportCode::BadQuote,
                            line,
                            "a carriage return that is not followed by a line feed",
                        ));
                    }
                    line += 1;
                    break;
                }
                Some('\n') => {
                    line += 1;
                    break;
                }
                _ => break,
            }
        }
        if fields.len() == 1 && fields[0].is_empty() && !first_quoted {
            continue;
        }
        match &header {
            None => {
                header = Some(fields.iter().map(|f| f.trim().to_lowercase()).collect());
                header_line = start;
            }
            Some(names) => {
                if fields.len() != names.len() {
                    return Err(Issue::new(
                        ImportCode::RowWidth,
                        start,
                        format!(
                            "the line has {} fields and the header has {}",
                            fields.len(),
                            names.len()
                        ),
                    ));
                }
                if rows.len() >= limits.max_rows {
                    return Err(Issue::new(
                        ImportCode::TooManyRows,
                        start,
                        format!("the file has more than {} data rows", limits.max_rows),
                    ));
                }
                rows.push(CsvRow {
                    line: start,
                    fields,
                });
            }
        }
    }
    match header {
        None => Err(Issue::new(
            ImportCode::BadHeader,
            0,
            "the file has no header line",
        )),
        Some(header) => Ok(CsvTable {
            header_line,
            header,
            rows,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LIMITS: CsvLimits = CsvLimits {
        max_bytes: 1000,
        max_rows: 10,
        max_columns: 5,
        max_field_bytes: 20,
    };

    fn ok(text: &str) -> CsvTable {
        parse(text.as_bytes(), ',', &LIMITS).unwrap()
    }

    fn err(text: &str) -> Issue {
        parse(text.as_bytes(), ',', &LIMITS).unwrap_err()
    }

    fn row(line: usize, fields: &[&str]) -> CsvRow {
        CsvRow {
            line,
            fields: fields.iter().map(|f| f.to_string()).collect(),
        }
    }

    #[test]
    fn lf_and_crlf_give_the_same_table_and_the_header_is_normalised() {
        let lf = ok(" Ref ,B,c\n1,2,3\n4,5,6\n");
        let crlf = ok(" Ref ,B,c\r\n1,2,3\r\n4,5,6\r\n");
        assert_eq!(lf, crlf);
        assert_eq!(lf.header, vec!["ref", "b", "c"]);
        assert_eq!(lf.header_line, 1);
        assert_eq!(
            lf.rows,
            vec![row(2, &["1", "2", "3"]), row(3, &["4", "5", "6"])]
        );
        assert_eq!(ok("a,b\n1,2").rows, vec![row(2, &["1", "2"])]);
    }

    #[test]
    fn a_byte_order_mark_is_ignored() {
        let mut bytes = vec![0xEF, 0xBB, 0xBF];
        bytes.extend_from_slice(b"a,b\n1,2\n");
        let with = parse(&bytes, ',', &LIMITS).unwrap();
        assert_eq!(with, ok("a,b\n1,2\n"));
        assert_eq!(with.header[0], "a");
    }

    #[test]
    fn quotes_hold_separators_quotes_and_line_breaks() {
        let t = ok("h1,h2,h3\n\"a,1\",\"say \"\"hi\"\"\",x\n\"l1\nl2\",b,c\nx,y,z\n");
        assert_eq!(t.rows[0], row(2, &["a,1", "say \"hi\"", "x"]));
        assert_eq!(t.rows[1], row(3, &["l1\nl2", "b", "c"]));
        assert_eq!(t.rows[2].line, 5);
    }

    #[test]
    fn empty_fields_and_blank_lines() {
        let t = ok("a,b,c\n,,\n\n1,,3\n\n");
        assert_eq!(t.rows, vec![row(2, &["", "", ""]), row(4, &["1", "", "3"])]);
        let t = ok("a,b\n\"\",\"\"\n");
        assert_eq!(t.rows, vec![row(2, &["", ""])]);
    }

    #[test]
    fn the_separator_is_a_parameter() {
        let t = parse(b"a;b\n1,5;\"x;y\"\n", ';', &LIMITS).unwrap();
        assert_eq!(t.rows, vec![row(2, &["1,5", "x;y"])]);
        let t = parse(b"a\tb\n1\t2\n", '\t', &LIMITS).unwrap();
        assert_eq!(t.rows, vec![row(2, &["1", "2"])]);
    }

    #[test]
    fn badly_placed_quotes_are_refused_with_their_line() {
        let e = err("a,b\n\"open,1\n2,3\n");
        assert_eq!((e.code, e.line), (ImportCode::BadQuote, 2));
        let e = err("a,b\n\"x\"y,1\n");
        assert_eq!((e.code, e.line), (ImportCode::BadQuote, 2));
        let e = err("a,b\n1,x\"y\n");
        assert_eq!((e.code, e.line), (ImportCode::BadQuote, 2));
        let e = err("a,b\n1, \"x\"\n");
        assert_eq!(e.code, ImportCode::BadQuote);
        let e = err("a,b\r1,2\r");
        assert_eq!((e.code, e.line), (ImportCode::BadQuote, 1));
    }

    #[test]
    fn a_file_that_is_not_utf8_names_the_line() {
        let e = parse(b"a,b\n1,\xFF\n", ',', &LIMITS).unwrap_err();
        assert_eq!((e.code, e.line), (ImportCode::BadEncoding, 2));
    }

    #[test]
    fn every_limit_is_enforced() {
        let e = parse(&[b'a'; 1001], ',', &LIMITS).unwrap_err();
        assert_eq!((e.code, e.line), (ImportCode::TooLarge, 0));
        let mut rows = String::from("a\n");
        for _ in 0..11 {
            rows.push_str("x\n");
        }
        let e = err(&rows);
        assert_eq!((e.code, e.line), (ImportCode::TooManyRows, 12));
        assert!(parse(&rows.as_bytes()[..rows.len() - 2], ',', &LIMITS).is_ok());
        let e = err("a,b,c,d,e,f\n");
        assert_eq!((e.code, e.line), (ImportCode::TooManyColumns, 1));
        let e = err("a\n123456789012345678901\n");
        assert_eq!((e.code, e.line), (ImportCode::FieldTooLong, 2));
        let e = err("a\n\"123456789012345678901\"\n");
        assert_eq!((e.code, e.line), (ImportCode::FieldTooLong, 2));
        assert!(parse(b"a\n12345678901234567890\n", ',', &LIMITS).is_ok());
    }

    #[test]
    fn rows_must_be_as_wide_as_the_header_and_there_must_be_a_header() {
        let e = err("a,b\n1,2\n3\n");
        assert_eq!((e.code, e.line), (ImportCode::RowWidth, 3));
        let e = err("a,b\n1,2,3\n");
        assert_eq!((e.code, e.line), (ImportCode::RowWidth, 2));
        assert_eq!(err("").code, ImportCode::BadHeader);
        assert_eq!(err("\n\r\n\n").code, ImportCode::BadHeader);
    }
}
