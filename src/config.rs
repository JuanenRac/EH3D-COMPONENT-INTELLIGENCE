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
}

impl Default for Config {
    fn default() -> Config {
        Config {
            max_database_bytes: 64 * 1024 * 1024,
            max_components: 200_000,
            max_observations: 64,
            require_licence_for_export: true,
            inferred_blocks_verified: true,
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

impl Config {
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
}
