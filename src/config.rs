// SPDX-License-Identifier: GPL-3.0-or-later
//! The settings of the library. Every tuned value is a line of the settings file; a missing line keeps the conservative default.

use crate::error::CintError;

/// The settings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    /// The most bytes a database file may have.
    pub max_database_bytes: u64,
    /// The most components a database may hold.
    pub max_components: usize,
    /// The most observations one field may hold.
    pub max_observations: usize,
    /// A record may be exported only when it records the licence of its data.
    pub require_licence_for_export: bool,
    /// An observation that is only `inferred` can never make a record verified.
    pub inferred_blocks_verified: bool,
    /// The most bytes a CSV file to import may have.
    pub csv_max_bytes: usize,
    /// The most data rows a CSV file may have.
    pub csv_max_rows: usize,
    /// The most columns a line of a CSV file may have.
    pub csv_max_columns: usize,
    /// The most bytes one field of a CSV file may have.
    pub csv_max_field_bytes: usize,
    /// The character that separates the fields of a CSV file.
    pub csv_separator: char,
    /// The most problems an import reports before it stops looking for more.
    pub csv_max_issues: usize,
    /// The licence written on the records an import creates; empty leaves them without one (so they cannot be exported).
    pub import_licence: String,
}

/// The columns an import needs: the number of columns a line must be allowed to have at least.
pub const CSV_MIN_COLUMNS: usize = 6;

impl Default for Config {
    fn default() -> Config {
        Config {
            max_database_bytes: 64 * 1024 * 1024,
            max_components: 200_000,
            max_observations: 64,
            require_licence_for_export: true,
            inferred_blocks_verified: true,
            csv_max_bytes: 16 * 1024 * 1024,
            csv_max_rows: 100_000,
            csv_max_columns: 16,
            csv_max_field_bytes: 4096,
            csv_separator: ',',
            csv_max_issues: 50,
            import_licence: String::new(),
        }
    }
}

fn flag(text: &str) -> Option<bool> {
    match text {
        "true" | "on" | "yes" => Some(true),
        "false" | "off" | "no" => Some(false),
        _ => None,
    }
}

fn positive(text: &str) -> Option<usize> {
    text.parse::<usize>().ok().filter(|n| *n > 0)
}

fn separator(text: &str) -> Option<char> {
    if text == "tab" {
        return Some('\t');
    }
    let mut chars = text.chars();
    match (chars.next(), chars.next()) {
        (Some(c), None) if !matches!(c, '"' | '\r' | '\n' | ' ') => Some(c),
        _ => None,
    }
}

fn separator_text(c: char) -> String {
    if c == '\t' {
        "tab".to_string()
    } else {
        c.to_string()
    }
}

impl Config {
    /// The text of the settings file with every key at its default. The example file of the repository is exactly this text.
    pub fn default_file() -> String {
        let d = Config::default();
        format!(
            "\
# Settings of EH3D-COMPONENT-INTELLIGENCE. A line that is missing keeps the value written here.
# Lines are `key = value`; a line that starts with # is a comment.

# Limits of a database file
max_database_bytes = {}
max_components = {}
max_observations = {}

# Rules of the checks
require_licence_for_export = {}
inferred_blocks_verified = {}

# Limits of a CSV file to import
csv_max_bytes = {}
csv_max_rows = {}
csv_max_columns = {}
csv_max_field_bytes = {}
csv_max_issues = {}

# The character between the fields of a CSV file: one character, or the word tab
csv_separator = {}

# The licence written on imported records; empty leaves them without one, and then they cannot be exported
import_licence = {}
",
            d.max_database_bytes,
            d.max_components,
            d.max_observations,
            d.require_licence_for_export,
            d.inferred_blocks_verified,
            d.csv_max_bytes,
            d.csv_max_rows,
            d.csv_max_columns,
            d.csv_max_field_bytes,
            d.csv_max_issues,
            separator_text(d.csv_separator),
            d.import_licence,
        )
        .replace("import_licence = \n", "import_licence =\n")
    }

    /// Parse a settings file (`key = value` lines, `#` comments).
    pub fn parse(text: &str) -> Result<Config, CintError> {
        let mut config = Config::default();
        for (index, raw) in text.lines().enumerate() {
            let line = index + 1;
            let raw = raw.trim();
            if raw.is_empty() || raw.starts_with('#') {
                continue;
            }
            let (key, value) = raw
                .split_once('=')
                .ok_or(CintError::ConfigSyntax { line })?;
            let (key, value) = (key.trim(), value.trim());
            let bad = || CintError::ConfigBadValue {
                line,
                key: key.to_string(),
            };
            match key {
                "max_database_bytes" => {
                    config.max_database_bytes = value.parse().map_err(|_| bad())?
                }
                "max_components" => {
                    let n: usize = value.parse().map_err(|_| bad())?;
                    if n == 0 {
                        return Err(bad());
                    }
                    config.max_components = n;
                }
                "max_observations" => {
                    let n: usize = value.parse().map_err(|_| bad())?;
                    if n == 0 {
                        return Err(bad());
                    }
                    config.max_observations = n;
                }
                "require_licence_for_export" => {
                    config.require_licence_for_export = flag(value).ok_or_else(bad)?
                }
                "inferred_blocks_verified" => {
                    config.inferred_blocks_verified = flag(value).ok_or_else(bad)?
                }
                "csv_max_bytes" => {
                    config.csv_max_bytes = positive(value).ok_or_else(bad)?;
                }
                "csv_max_rows" => {
                    config.csv_max_rows = positive(value).ok_or_else(bad)?;
                }
                "csv_max_columns" => {
                    let n = positive(value).ok_or_else(bad)?;
                    if n < CSV_MIN_COLUMNS {
                        return Err(bad());
                    }
                    config.csv_max_columns = n;
                }
                "csv_max_field_bytes" => {
                    config.csv_max_field_bytes = positive(value).ok_or_else(bad)?;
                }
                "csv_max_issues" => {
                    config.csv_max_issues = positive(value).ok_or_else(bad)?;
                }
                "csv_separator" => config.csv_separator = separator(value).ok_or_else(bad)?,
                "import_licence" => config.import_licence = value.to_string(),
                _ => {
                    return Err(CintError::ConfigUnknownKey {
                        line,
                        key: key.to_string(),
                    })
                }
            }
        }
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_conservative() {
        let c = Config::parse("").unwrap();
        assert!(c.require_licence_for_export && c.inferred_blocks_verified);
        assert_eq!(c.max_observations, 64);
    }

    #[test]
    fn lines_change_values_and_bad_lines_are_refused() {
        let c =
            Config::parse("# c\nmax_components = 5\nrequire_licence_for_export = off\n").unwrap();
        assert_eq!(c.max_components, 5);
        assert!(!c.require_licence_for_export);
        assert!(matches!(
            Config::parse("nonsense\n"),
            Err(CintError::ConfigSyntax { line: 1 })
        ));
        assert!(matches!(
            Config::parse("a = 1\n"),
            Err(CintError::ConfigUnknownKey { line: 1, .. })
        ));
        assert!(matches!(
            Config::parse("max_components = 0\n"),
            Err(CintError::ConfigBadValue { line: 1, .. })
        ));
        assert!(matches!(
            Config::parse("inferred_blocks_verified = maybe\n"),
            Err(CintError::ConfigBadValue { line: 1, .. })
        ));
    }

    #[test]
    fn the_default_file_says_the_defaults_and_the_example_is_that_file() {
        let text = Config::default_file();
        assert_eq!(Config::parse(&text).unwrap(), Config::default());
        assert_eq!(include_str!("../examples/settings.cfg"), text);
        assert!(text.contains("csv_separator = ,\n"));
        assert!(text.contains("csv_max_columns = 16\n"));
    }

    #[test]
    fn the_csv_settings_are_read_and_bad_ones_refused() {
        let c = Config::parse(
            "csv_separator = tab\ncsv_max_rows = 7\ncsv_max_columns = 6\nimport_licence = CC0 data\n",
        )
        .unwrap();
        assert_eq!(c.csv_separator, '\t');
        assert_eq!((c.csv_max_rows, c.csv_max_columns), (7, 6));
        assert_eq!(c.import_licence, "CC0 data");
        assert_eq!(
            Config::parse("csv_separator = ;\n").unwrap().csv_separator,
            ';'
        );
        assert_eq!(
            Config::parse("csv_separator = #\n").unwrap().csv_separator,
            '#'
        );
        for bad in [
            "csv_separator = ab\n",
            "csv_separator = \"\n",
            "csv_separator =\n",
            "csv_max_bytes = 0\n",
            "csv_max_rows = -1\n",
            "csv_max_columns = 5\n",
            "csv_max_field_bytes = x\n",
            "csv_max_issues = 0\n",
        ] {
            assert!(
                matches!(
                    Config::parse(bad),
                    Err(CintError::ConfigBadValue { line: 1, .. })
                ),
                "{bad}"
            );
        }
    }
}
