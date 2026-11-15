// SPDX-License-Identifier: GPL-3.0-or-later
//! The simulation mode: a host twin of the program that exports a list of parts, with the damaged files it can produce and the recovery from them.
//!
//! The lists of parts come from other programs (the design tool, a supplier), which do not run here. [`ExportTwin`] stands in for them: given a seed it
//! writes the same list every time (it never reads the clock), so the import, the refusals with their line and code, the limits of the reader and the safe
//! writing of the database can be exercised against every kind of damage ([`Fault`]) and against the recovery ([`import_with_reexport`]). Nothing here has
//! run against a real export.

use crate::config::Config;
use crate::import::{import_csv, Imported, COLUMNS};
use crate::issue::Issue;

/// A small deterministic generator (SplitMix64): the same seed always gives the same numbers.
#[derive(Debug, Clone)]
pub struct SimRng(u64);

impl SimRng {
    /// A generator that starts from `seed`.
    pub fn new(seed: u64) -> SimRng {
        SimRng(seed)
    }

    /// The next number.
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A number below `bound` (`bound` must not be 0).
    pub fn below(&mut self, bound: u64) -> u64 {
        self.next_u64() % bound
    }
}

/// The damage a file can have when it reaches the import.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fault {
    /// A byte that is not UTF-8.
    InvalidUtf8,
    /// The `datasheet` column is missing.
    MissingColumn,
    /// A column this version does not know.
    ExtraColumn,
    /// A line with one field less than the header.
    RaggedRow,
    /// Two lines use the same reference.
    DuplicateReference,
    /// A line without a part number.
    EmptyPartNumber,
    /// A quotation mark that is opened and never closed.
    UnclosedQuote,
}

/// A list of parts as a table, before it is written as text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Export {
    /// The names of the columns.
    pub header: Vec<String>,
    /// The lines.
    pub rows: Vec<Vec<String>>,
}

fn quote(field: &str, separator: char) -> String {
    if field.contains([separator, '"', '\n', '\r']) {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
}

impl Export {
    /// The file as bytes, with `separator` between the fields and a line feed after each line.
    pub fn to_csv(&self, separator: char) -> Vec<u8> {
        let mut out = String::new();
        for line in std::iter::once(&self.header).chain(&self.rows) {
            let fields: Vec<String> = line.iter().map(|f| quote(f, separator)).collect();
            out.push_str(&fields.join(&separator.to_string()));
            out.push('\n');
        }
        out.into_bytes()
    }

    /// How many different parts the lines make (the same manufacturer and part number, whatever the case, is one part).
    pub fn parts(&self) -> usize {
        let at = |name: &str| self.header.iter().position(|h| h == name);
        let (Some(m), Some(p)) = (at("manufacturer"), at("mpn")) else {
            return 0;
        };
        self.rows
            .iter()
            .map(|r| (r[m].trim().to_lowercase(), r[p].trim().to_lowercase()))
            .collect::<std::collections::BTreeSet<_>>()
            .len()
    }
}

/// The twin of the program that exports the list.
#[derive(Debug, Clone)]
pub struct ExportTwin {
    rng: SimRng,
}

impl ExportTwin {
    /// A twin that writes the same lists for the same seed.
    pub fn new(seed: u64) -> ExportTwin {
        ExportTwin {
            rng: SimRng::new(seed),
        }
    }

    /// A list of `lines` parts with unique references. Some manufacturers have a comma in their name, so their field is quoted.
    pub fn export(&mut self, lines: usize) -> Export {
        const MAKERS: [&str; 4] = ["Yageo", "Murata", "Texas Instruments, Inc.", "Vishay"];
        const PACKAGES: [&str; 3] = ["0402", "0603", "SOT-23-5"];
        let rows = (0..lines)
            .map(|n| {
                let value = format!("{} nF", 1 + self.rng.below(470));
                let package = PACKAGES[self.rng.below(3) as usize].to_string();
                let maker = MAKERS[self.rng.below(4) as usize].to_string();
                let mpn = format!("PN-{:04}", self.rng.below(40));
                let sheet = if self.rng.below(2) == 0 {
                    String::new()
                } else {
                    format!("https://example.invalid/{mpn}.pdf")
                };
                vec![format!("R{}", n + 1), value, package, maker, mpn, sheet]
            })
            .collect();
        Export {
            header: COLUMNS.iter().map(|c| c.to_string()).collect(),
            rows,
        }
    }
}

/// The bytes of the file with the damage applied. Returns `None` when the list is too short for the damage (at least two lines are needed).
pub fn damage(export: &Export, fault: Fault, separator: char) -> Option<Vec<u8>> {
    if export.rows.len() < 2 {
        return None;
    }
    let mut table = export.clone();
    match fault {
        Fault::InvalidUtf8 => {
            let mut bytes = export.to_csv(separator);
            let middle = bytes.len() / 2;
            bytes.insert(middle, 0xFF);
            return Some(bytes);
        }
        Fault::UnclosedQuote => {
            let mut bytes = export.to_csv(separator);
            bytes.extend_from_slice(b"R999,\"never closed");
            return Some(bytes);
        }
        Fault::MissingColumn => {
            let at = table.header.iter().position(|h| h == "datasheet")?;
            table.header.remove(at);
            for row in &mut table.rows {
                row.remove(at);
            }
        }
        Fault::ExtraColumn => {
            table.header.push("colour".into());
            for row in &mut table.rows {
                row.push("red".into());
            }
        }
        Fault::RaggedRow => {
            table.rows[1].pop();
        }
        Fault::DuplicateReference => {
            table.rows[1][0] = table.rows[0][0].clone();
        }
        Fault::EmptyPartNumber => {
            table.rows[0][4].clear();
        }
    }
    Some(table.to_csv(separator))
}

/// The recovery: when a file is refused, ask the exporting program for the list again (a new file from the twin) until one is accepted, up to `attempts` times.
/// Returns what the last attempt gave and how many files were read. The database is never written here.
pub fn import_with_reexport(
    first: &[u8],
    twin: &mut ExportTwin,
    lines: usize,
    attempts: usize,
    config: &Config,
) -> (Result<Imported, Vec<Issue>>, usize) {
    let mut result = import_csv(first, "export.csv", config);
    let mut read = 1;
    while result.is_err() && read < attempts {
        let again = twin.export(lines).to_csv(config.csv_separator);
        result = import_csv(&again, "export.csv", config);
        read += 1;
    }
    (result, read)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::issue::ImportCode;
    use crate::store::write_atomic;

    fn config() -> Config {
        Config::default()
    }

    fn sep() -> char {
        config().csv_separator
    }

    fn codes(bytes: &[u8]) -> Vec<ImportCode> {
        import_csv(bytes, "export.csv", &config())
            .unwrap_err()
            .iter()
            .map(|i| i.code)
            .collect()
    }

    #[test]
    fn the_same_seed_gives_the_same_file_and_another_seed_a_different_one() {
        let a = ExportTwin::new(3).export(30).to_csv(sep());
        assert_eq!(a, ExportTwin::new(3).export(30).to_csv(sep()));
        assert_ne!(a, ExportTwin::new(4).export(30).to_csv(sep()));
    }

    #[test]
    fn a_file_made_by_the_twin_is_imported_whole() {
        for seed in 0..10 {
            let export = ExportTwin::new(seed).export(60);
            let imported = import_csv(&export.to_csv(sep()), "export.csv", &config()).unwrap();
            assert_eq!(imported.rows, 60, "seed {seed}");
            assert_eq!(imported.database.components.len(), export.parts());
        }
    }

    #[test]
    fn the_database_does_not_depend_on_the_order_of_the_lines() {
        let export = ExportTwin::new(12).export(50);
        let mut reversed = export.clone();
        reversed.rows.reverse();
        let a = import_csv(&export.to_csv(sep()), "export.csv", &config()).unwrap();
        let b = import_csv(&reversed.to_csv(sep()), "export.csv", &config()).unwrap();
        assert_eq!(
            a.database.to_canonical_json().unwrap(),
            b.database.to_canonical_json().unwrap()
        );
    }

    #[test]
    fn every_damage_is_refused_with_its_code() {
        let export = ExportTwin::new(1).export(10);
        let cases = [
            (Fault::InvalidUtf8, ImportCode::BadEncoding),
            (Fault::MissingColumn, ImportCode::BadHeader),
            (Fault::ExtraColumn, ImportCode::UnknownColumn),
            (Fault::RaggedRow, ImportCode::RowWidth),
            (Fault::DuplicateReference, ImportCode::DuplicateReference),
            (Fault::EmptyPartNumber, ImportCode::EmptyRequired),
            (Fault::UnclosedQuote, ImportCode::BadQuote),
        ];
        for (fault, code) in cases {
            let bytes = damage(&export, fault, sep()).unwrap();
            assert!(codes(&bytes).contains(&code), "{fault:?}");
        }
    }

    #[test]
    fn a_list_too_short_for_the_damage_gives_no_file() {
        let export = ExportTwin::new(1).export(1);
        assert!(damage(&export, Fault::RaggedRow, sep()).is_none());
    }

    #[test]
    fn the_refusal_names_the_line_of_the_damage() {
        let export = ExportTwin::new(2).export(10);
        let bytes = damage(&export, Fault::RaggedRow, sep()).unwrap();
        let issues = import_csv(&bytes, "export.csv", &config()).unwrap_err();
        // the header is line 1, so the second data line is line 3
        assert_eq!((issues[0].code, issues[0].line), (ImportCode::RowWidth, 3));
    }

    #[test]
    fn the_size_limit_holds_to_the_byte() {
        let bytes = ExportTwin::new(6).export(20).to_csv(sep());
        let mut exact = config();
        exact.csv_max_bytes = bytes.len();
        assert!(import_csv(&bytes, "export.csv", &exact).is_ok());
        exact.csv_max_bytes = bytes.len() - 1;
        let issues = import_csv(&bytes, "export.csv", &exact).unwrap_err();
        assert_eq!(issues[0].code, ImportCode::TooLarge);
    }

    #[test]
    fn a_damaged_byte_never_panics_and_always_gives_the_same_answer() {
        let good = ExportTwin::new(8).export(25).to_csv(sep());
        let mut rng = SimRng::new(2024);
        for _ in 0..300 {
            let mut bytes = good.clone();
            let at = rng.below(bytes.len() as u64) as usize;
            bytes[at] = rng.below(256) as u8;
            let first = import_csv(&bytes, "export.csv", &config());
            let second = import_csv(&bytes, "export.csv", &config());
            assert_eq!(first, second);
        }
    }

    #[test]
    fn a_large_list_within_the_limits_is_imported() {
        let export = ExportTwin::new(15).export(3000);
        let imported = import_csv(&export.to_csv(sep()), "export.csv", &config()).unwrap();
        assert_eq!(imported.rows, 3000);
        assert_eq!(imported.database.components.len(), export.parts());
    }

    #[test]
    fn asking_for_the_list_again_recovers_from_a_damaged_file() {
        let export = ExportTwin::new(4).export(40);
        let bad = damage(&export, Fault::InvalidUtf8, sep()).unwrap();
        let mut twin = ExportTwin::new(5);
        let (result, read) = import_with_reexport(&bad, &mut twin, 40, 3, &config());
        assert_eq!(read, 2);
        assert_eq!(result.unwrap().rows, 40);
        // a file that is already good is read once
        let good = export.to_csv(sep());
        let (result, read) = import_with_reexport(&good, &mut twin, 40, 3, &config());
        assert_eq!((read, result.is_ok()), (1, true));
        // and the recovery gives up after the attempts it was given
        let (result, read) = import_with_reexport(&bad, &mut twin, 40, 1, &config());
        assert_eq!((read, result.is_err()), (1, true));
    }

    #[test]
    fn the_database_file_is_written_whole_once_and_never_replaced_by_mistake() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("components.json");
        let imported = import_csv(
            &ExportTwin::new(9).export(15).to_csv(sep()),
            "export.csv",
            &config(),
        )
        .unwrap();
        let text = imported.database.to_canonical_json().unwrap();
        write_atomic(&path, &text, false).unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), text);
        // a second write without the explicit order is refused and the file is untouched
        let refused = write_atomic(&path, "other", false).unwrap_err();
        assert_eq!(refused.code, ImportCode::OutputExists);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), text);
        // with the order it is replaced
        write_atomic(&path, "other", true).unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "other");
        // a folder that does not exist is a failed write, and nothing is left behind
        let missing = dir.path().join("missing").join("components.json");
        assert_eq!(
            write_atomic(&missing, &text, false).unwrap_err().code,
            ImportCode::WriteFailed
        );
        let left: Vec<_> = std::fs::read_dir(dir.path())
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        assert_eq!(left, vec!["components.json".to_string()]);
    }
}
