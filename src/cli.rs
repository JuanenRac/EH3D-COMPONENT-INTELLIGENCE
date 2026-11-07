// SPDX-License-Identifier: GPL-3.0-or-later
//! The command line. It is a thin layer: parsing the arguments and printing; the checks live in the other modules.
//!
//! Exit codes: 0 everything is as expected, 1 the database has findings (or an export was refused), 2 the input or the arguments are unusable.

use crate::config::Config;
use crate::database::{Database, Finding, Status};
use crate::error::CintError;
use crate::import::{import_csv, Imported};
use crate::issue::{ImportCode, Issue};
use crate::store::write_atomic;
use std::fs;
use std::io::Write;
use std::path::Path;

const USAGE: &str = "\
eh3d-component-intelligence [--config FILE] <command>

  check <database.json>                    list what is wrong: missing sources, conflicts, pending fields, missing licences
  show <database.json> <manufacturer> <part>
                                           print one component with the observations of every field
  export <database.json>                   print the database for redistribution; refused while a record has no licence
  list <database.json>                     one line per component: manufacturer, part number, status, references
  check-csv <parts.csv>                    read a CSV list of parts and report every problem; writes nothing
  import <parts.csv> <database.json> [--overwrite]
                                           make a database file from a CSV list of parts; an existing file is
                                           replaced only with --overwrite

Exit codes: 0 nothing found, 1 findings or a refused export, 2 unusable input (a CSV problem prints its EH3D-CINT code and line).
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
    match dispatch(args, out, err) {
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

fn print_issues(issues: &[Issue], err: &mut dyn Write) {
    for issue in issues {
        let _ = writeln!(err, "{issue}");
    }
    let _ = writeln!(err, "# {} problems, nothing was imported", issues.len());
}

/// Read a CSV file and turn it into a database. A problem of the file is printed on `err` and gives `Ok(None)`; an unreadable file is an error.
fn read_csv(
    path: &str,
    config: &Config,
    err: &mut dyn Write,
) -> Result<Option<Imported>, CintError> {
    let path = Path::new(path);
    let io = |source| CintError::Io {
        path: path.to_path_buf(),
        source,
    };
    if fs::metadata(path).map_err(io)?.len() > config.csv_max_bytes as u64 {
        let issue = Issue::new(
            ImportCode::TooLarge,
            0,
            format!("the file is larger than {} bytes", config.csv_max_bytes),
        );
        print_issues(&[issue], err);
        return Ok(None);
    }
    let bytes = fs::read(path).map_err(io)?;
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    match import_csv(&bytes, &name, config) {
        Ok(imported) => Ok(Some(imported)),
        Err(issues) => {
            print_issues(&issues, err);
            Ok(None)
        }
    }
}

fn dispatch(
    args: &[String],
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> Result<i32, Option<CintError>> {
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
        ["list", _] => {
            let db = load(&args[1])?;
            for c in &db.components {
                let _ = writeln!(
                    out,
                    "{} {} [{}] references: {}",
                    c.manufacturer,
                    c.part_number,
                    match c.status {
                        Status::Draft => "draft",
                        Status::Verified => "verified",
                    },
                    if c.references.is_empty() {
                        "-".to_string()
                    } else {
                        c.references.join(" ")
                    }
                );
            }
            let _ = writeln!(out, "# {} components", db.components.len());
            Ok(0)
        }
        ["check-csv", _] => match read_csv(&args[1], &config, err).map_err(Some)? {
            Some(imported) => {
                let _ = writeln!(
                    out,
                    "# {} rows, {} components, no problems",
                    imported.rows,
                    imported.database.components.len()
                );
                Ok(0)
            }
            None => Ok(2),
        },
        ["import", _, _] | ["import", _, _, "--overwrite"] => {
            let overwrite = args.len() == 4;
            let Some(imported) = read_csv(&args[1], &config, err).map_err(Some)? else {
                return Ok(2);
            };
            let text = imported.database.to_canonical_json().map_err(Some)?;
            if let Err(issue) = write_atomic(Path::new(&args[2]), &text, overwrite) {
                let _ = writeln!(err, "{issue}");
                return Ok(2);
            }
            let _ = writeln!(
                out,
                "# {} rows, {} components written to {}",
                imported.rows,
                imported.database.components.len(),
                args[2]
            );
            Ok(0)
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

    fn example(name: &str) -> String {
        format!("{}/examples/{name}", env!("CARGO_MANIFEST_DIR"))
    }

    #[test]
    fn the_examples_are_read_whatever_their_encoding_and_line_endings() {
        let (code, text, _) = call(&["check-csv", &example("parts-small.csv")]);
        assert_eq!(code, 0);
        assert!(text.contains("# 4 rows, 3 components, no problems"));
        let (code, text, _) = call(&["check-csv", &example("parts-quotes-crlf.csv")]);
        assert_eq!(code, 0);
        assert!(text.contains("# 3 rows, 3 components, no problems"));
        let (code, text, _) = call(&[
            "--config",
            &example("semicolon.cfg"),
            "check-csv",
            &example("parts-semicolon.csv"),
        ]);
        assert_eq!(code, 0);
        assert!(text.contains("# 2 rows, 2 components, no problems"));
        let (code, _, err) = call(&["check-csv", &example("parts-semicolon.csv")]);
        assert_eq!(code, 2);
        assert!(err.contains("EH3D-CINT-109"));
        let (code, text, _) = call(&[
            "--config",
            &example("settings.cfg"),
            "check-csv",
            &example("parts-small.csv"),
        ]);
        assert_eq!(code, 0);
        assert!(text.contains("4 rows"));
    }

    #[test]
    fn import_writes_the_database_and_check_and_list_read_it_back() {
        let work = tempfile::tempdir().unwrap();
        let out_file = work.path().join("parts.json");
        let out_path = out_file.to_str().unwrap();
        let (code, text, _) = call(&[
            "--config",
            &example("semicolon.cfg"),
            "import",
            &example("parts-semicolon.csv"),
            out_path,
        ]);
        assert_eq!(code, 0);
        assert!(text.contains("# 2 rows, 2 components written to"));
        let written = fs::read_to_string(&out_file).unwrap();
        assert!(written.ends_with("}\n"));
        assert!(written.contains("\"source\": \"parts-semicolon.csv\""));
        assert!(written.contains("\"licence\": \"own notes\""));
        let (code, text, _) = call(&["check", out_path]);
        assert_eq!(code, 0, "{text}");
        let (code, text, _) = call(&["list", out_path]);
        assert_eq!(code, 0);
        assert_eq!(
            text,
            "Yageo RC0402FR-071K5L [draft] references: R2\n\
Yageo RC0402FR-074K7L [draft] references: R1\n\
# 2 components\n"
        );
    }

    #[test]
    fn without_a_licence_in_the_settings_the_imported_data_is_flagged_and_not_exported() {
        let work = tempfile::tempdir().unwrap();
        let out_file = work.path().join("parts.json");
        let out_path = out_file.to_str().unwrap();
        assert_eq!(
            call(&["import", &example("parts-small.csv"), out_path]).0,
            0
        );
        let (code, text, _) = call(&["check", out_path]);
        assert_eq!(code, 1);
        assert_eq!(text.matches("PHOENIX-CINT-006").count(), 3);
        assert_eq!(call(&["export", out_path]).0, 1);
        let (_, text, _) = call(&["list", out_path]);
        assert!(text.contains("Yageo RC0603FR-0710KL [draft] references: R1 R2\n"));
    }

    #[test]
    fn an_existing_database_is_replaced_only_with_the_explicit_order() {
        let work = tempfile::tempdir().unwrap();
        let out_file = work.path().join("parts.json");
        let out_path = out_file.to_str().unwrap();
        let small = example("parts-small.csv");
        assert_eq!(call(&["import", &small, out_path]).0, 0);
        let first = fs::read_to_string(&out_file).unwrap();
        let other = example("parts-quotes-crlf.csv");
        let (code, _, err) = call(&["import", &other, out_path]);
        assert_eq!(code, 2);
        assert!(err.contains("EH3D-CINT-112"));
        assert_eq!(fs::read_to_string(&out_file).unwrap(), first);
        assert_eq!(call(&["import", &small, out_path, "--overwrite"]).0, 0);
        assert_eq!(fs::read_to_string(&out_file).unwrap(), first);
        assert_eq!(call(&["import", &other, out_path, "--overwrite"]).0, 0);
        assert_ne!(fs::read_to_string(&out_file).unwrap(), first);
    }

    #[test]
    fn a_bad_csv_lists_its_problems_with_code_and_line_and_writes_nothing() {
        let work = tempfile::tempdir().unwrap();
        let bad = work.path().join("bad.csv");
        let out_file = work.path().join("parts.json");
        let head = "reference,value,package,manufacturer,mpn,datasheet\n";
        fs::write(&bad, format!("{head}R1,1,a,M,P,\nR1,1,a,M,Q,\n,1,a,M,Q,\n")).unwrap();
        let (code, _, err) = call(&["import", bad.to_str().unwrap(), out_file.to_str().unwrap()]);
        assert_eq!(code, 2);
        assert!(err.contains("EH3D-CINT-111 line 3:"));
        assert!(err.contains("EH3D-CINT-110 line 4:"));
        assert!(err.contains("# 2 problems, nothing was imported"));
        assert!(!out_file.exists());
        fs::write(&bad, format!("{head}R1,\"open,a,M,P,\n")).unwrap();
        let (code, _, err) = call(&["check-csv", bad.to_str().unwrap()]);
        assert_eq!(code, 2);
        assert!(err.contains("EH3D-CINT-106 line 2:"));
        fs::write(&bad, b"reference,value\n\xFF\n").unwrap();
        let (code, _, err) = call(&["check-csv", bad.to_str().unwrap()]);
        assert_eq!(code, 2);
        assert!(err.contains("EH3D-CINT-105 line 2:"));
    }

    #[test]
    fn the_limits_of_the_settings_file_apply_to_the_csv() {
        let work = tempfile::tempdir().unwrap();
        let cfg = work.path().join("limits.cfg");
        let small = example("parts-small.csv");
        fs::write(&cfg, "csv_max_rows = 1\n").unwrap();
        let (code, _, err) = call(&["--config", cfg.to_str().unwrap(), "check-csv", &small]);
        assert_eq!(code, 2);
        assert!(err.contains("EH3D-CINT-102 line 3:"));
        fs::write(&cfg, "csv_max_bytes = 20\n").unwrap();
        let (code, _, err) = call(&["--config", cfg.to_str().unwrap(), "check-csv", &small]);
        assert_eq!(code, 2);
        assert!(err.contains("EH3D-CINT-101"));
        fs::write(&cfg, "csv_max_field_bytes = 12\n").unwrap();
        let (code, _, err) = call(&["--config", cfg.to_str().unwrap(), "check-csv", &small]);
        assert_eq!(code, 2);
        assert!(err.contains("EH3D-CINT-104 line 2:"));
    }

    #[test]
    fn the_new_commands_refuse_bad_arguments_and_missing_files() {
        assert_eq!(call(&["import", "a.csv"]).0, 2);
        assert_eq!(call(&["import", "a.csv", "b.json", "--force"]).0, 2);
        assert_eq!(call(&["check-csv"]).0, 2);
        assert_eq!(call(&["list"]).0, 2);
        let (code, _, err) = call(&["check-csv", "no-such-file.csv"]);
        assert_eq!(code, 2);
        assert!(err.contains("cannot read"));
        assert_eq!(call(&["list", "no-such-file.json"]).0, 2);
    }
}
