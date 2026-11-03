// SPDX-License-Identifier: GPL-3.0-or-later
//! The command line. It is a thin layer: parsing the arguments and printing; the checks live in the other modules.
//!
//! Exit codes: 0 everything is as expected, 1 the database has findings (or an export was refused), 2 the input or the arguments are unusable.

use crate::config::Config;
use crate::database::{Database, Finding};
use crate::error::CintError;
use std::fs;
use std::io::Write;
use std::path::Path;

const USAGE: &str = "\
eh3d-component-intelligence [--config FILE] <command>

  check <database.json>                    list what is wrong: missing sources, conflicts, pending fields, missing licences
  show <database.json> <manufacturer> <part>
                                           print one component with the observations of every field
  export <database.json>                   print the database for redistribution; refused while a record has no licence

Exit codes: 0 nothing found, 1 findings or a refused export, 2 unusable input.
";

fn read_text(path: &Path, limit: u64) -> Result<String, CintError> {
    let io = |source| CintError::Io {
        path: path.to_path_buf(),
        source,
    };
    if fs::metadata(path).map_err(io)?.len() > limit {
        return Err(CintError::TooLarge { what: "input file" });
    }
    fs::read_to_string(path).map_err(io)
}

fn print_findings(findings: &[Finding], out: &mut dyn Write) {
    for f in findings {
        let _ = writeln!(
            out,
            "{} {} {}{}: {}",
            f.code.as_str(),
            f.manufacturer,
            f.part_number,
            f.field
                .as_ref()
                .map(|n| format!(" [{n}]"))
                .unwrap_or_default(),
            f.message
        );
    }
}

/// Run the command line with the given arguments (without the program name). Returns the exit code.
pub fn run(args: &[String], out: &mut dyn Write, err: &mut dyn Write) -> i32 {
    match dispatch(args, out) {
        Ok(code) => code,
        Err(None) => {
            let _ = err.write_all(USAGE.as_bytes());
            2
        }
        Err(Some(error)) => {
            let _ = writeln!(err, "error: {error}");
            2
        }
    }
}

fn dispatch(args: &[String], out: &mut dyn Write) -> Result<i32, Option<CintError>> {
    let mut args = args.to_vec();
    let mut config = Config::default();
    if args.first().map(String::as_str) == Some("--config") {
        if args.len() < 3 {
            return Err(None);
        }
        config = Config::parse(&read_text(Path::new(&args[1]), 1024 * 1024).map_err(Some)?)
            .map_err(Some)?;
        args.drain(..2);
    }
    let load = |path: &String| -> Result<Database, Option<CintError>> {
        let text = read_text(Path::new(path), config.max_database_bytes).map_err(Some)?;
        Database::from_json(&text, &config).map_err(Some)
    };
    let rest: Vec<&str> = args.iter().map(String::as_str).collect();
    match rest.as_slice() {
        ["check", _] => {
            let db = load(&args[1])?;
            let findings = db.check(&config);
            print_findings(&findings, out);
            let _ = writeln!(
                out,
                "# {} components, {} findings",
                db.components.len(),
                findings.len()
            );
            Ok(if findings.is_empty() { 0 } else { 1 })
        }
        ["show", _, manufacturer, part] => {
            let db = load(&args[1])?;
            match db.find(manufacturer, part) {
                Some(c) => {
                    let text = serde_json::to_string_pretty(c)
                        .map_err(|e| Some(CintError::BadJson(e.to_string())))?;
                    let _ = writeln!(out, "{text}");
                    Ok(0)
                }
                None => Ok(1),
            }
        }
        ["export", _] => {
            let db = load(&args[1])?;
            match db.export(&config) {
                Ok(text) => {
                    let _ = writeln!(out, "{text}");
                    Ok(0)
                }
                Err(findings) => {
                    print_findings(&findings, out);
                    Ok(1)
                }
            }
        }
        _ => Err(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn call(list: &[&str]) -> (i32, String, String) {
        let args: Vec<String> = list.iter().map(|s| s.to_string()).collect();
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let code = run(&args, &mut out, &mut err);
        (
            code,
            String::from_utf8(out).unwrap(),
            String::from_utf8(err).unwrap(),
        )
    }

    const GOOD: &str = r#"{"components":[{"manufacturer":"Acme","part_number":"X1","origin":{"kind":"manual"},
        "licence":"own notes","fields":{"vcc_max":[{"value":"3.6 V","method":"transcribed","source":"data sheet"}]}}]}"#;
    const BAD: &str = r#"{"components":[{"manufacturer":"Acme","part_number":"X2","origin":{"kind":"manual"},
        "fields":{"vcc_max":[{"value":"3.6 V","method":"transcribed","source":"a"},{"value":"5 V","method":"transcribed","source":"b"}]}}]}"#;

    #[test]
    fn check_show_and_export_exit_codes() {
        let work = tempfile::tempdir().unwrap();
        let good = work.path().join("good.json");
        let bad = work.path().join("bad.json");
        fs::write(&good, GOOD).unwrap();
        fs::write(&bad, BAD).unwrap();
        let (good, bad) = (good.to_str().unwrap(), bad.to_str().unwrap());
        let (code, text, _) = call(&["check", good]);
        assert_eq!(code, 0);
        assert!(text.contains("1 components, 0 findings"));
        let (code, text, _) = call(&["check", bad]);
        assert_eq!(code, 1);
        assert!(text.contains("PHOENIX-CINT-003") && text.contains("PHOENIX-CINT-006"));
        assert_eq!(call(&["show", good, "acme", "x1"]).0, 0);
        assert_eq!(call(&["show", good, "acme", "nope"]).0, 1);
        assert_eq!(call(&["export", good]).0, 0);
        let (code, text, _) = call(&["export", bad]);
        assert_eq!(code, 1);
        assert!(text.contains("PHOENIX-CINT-006") && !text.contains("\"components\""));
    }

    #[test]
    fn bad_input_and_bad_arguments_exit_2() {
        let work = tempfile::tempdir().unwrap();
        let broken = work.path().join("broken.json");
        fs::write(&broken, "{").unwrap();
        let (code, _, err) = call(&["check", broken.to_str().unwrap()]);
        assert_eq!(code, 2);
        assert!(err.contains("cannot be read"));
        assert_eq!(call(&[]).0, 2);
        assert_eq!(call(&["check"]).0, 2);
        assert_eq!(call(&["nothing", "x"]).0, 2);
        assert_eq!(call(&["check", "no-such-file.json"]).0, 2);
        assert_eq!(call(&["--config", "x"]).0, 2);
    }

    #[test]
    fn a_settings_file_changes_the_export_rule() {
        let work = tempfile::tempdir().unwrap();
        let bad = work.path().join("bad.json");
        let cfg = work.path().join("c.cfg");
        fs::write(&bad, BAD).unwrap();
        fs::write(&cfg, "require_licence_for_export = off\n").unwrap();
        let (code, text, _) = call(&[
            "--config",
            cfg.to_str().unwrap(),
            "export",
            bad.to_str().unwrap(),
        ]);
        assert_eq!(code, 0);
        assert!(text.contains("\"components\""));
    }
}
