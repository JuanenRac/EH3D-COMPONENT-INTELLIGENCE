// SPDX-License-Identifier: GPL-3.0-or-later
//! Writing a database file. The storage form is a plain JSON file kept next to the project (see `docs/DATABASE_FORMAT.md`): no database server.
//!
//! The file is written whole to a temporary file in the same folder, flushed to the disk and then put in place, so a failure never leaves a
//! half-written database. An existing file is replaced only when the caller says so explicitly.

use crate::issue::{ImportCode, Issue};
use std::fs;
use std::io::Write;
use std::path::Path;

fn exists(path: &Path) -> Issue {
    Issue::new(
        ImportCode::OutputExists,
        0,
        format!(
            "{} already exists; nothing was written (use --overwrite to replace it)",
            path.display()
        ),
    )
}

/// Write `text` to `path` atomically. With `overwrite` false an existing file is never touched.
pub fn write_atomic(path: &Path, text: &str, overwrite: bool) -> Result<(), Issue> {
    let fail = |e: std::io::Error| {
        Issue::new(
            ImportCode::WriteFailed,
            0,
            format!("cannot write {}: {e}", path.display()),
        )
    };
    if !overwrite && path.exists() {
        return Err(exists(path));
    }
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .ok_or_else(|| {
            fail(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "the path has no file name",
            ))
        })?;
    let folder = match path.parent() {
        Some(p) if !p.as_os_str().is_empty() => p,
        _ => Path::new("."),
    };
    let temporary = folder.join(format!(".{name}.{}.tmp", std::process::id()));
    let written = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .and_then(|mut file| {
            file.write_all(text.as_bytes())?;
            file.sync_all()
        });
    if let Err(e) = written {
        let _ = fs::remove_file(&temporary);
        return Err(fail(e));
    }
    let placed = if overwrite {
        fs::rename(&temporary, path)
    } else {
        // A hard link fails when the name exists, so a file created meanwhile is never replaced.
        match fs::hard_link(&temporary, path) {
            Ok(()) => {
                let _ = fs::remove_file(&temporary);
                Ok(())
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                let _ = fs::remove_file(&temporary);
                return Err(exists(path));
            }
            Err(_) if path.exists() => {
                let _ = fs::remove_file(&temporary);
                return Err(exists(path));
            }
            // A file system without hard links: move the file after the check above.
            Err(_) => fs::rename(&temporary, path),
        }
    };
    placed.map_err(|e| {
        let _ = fs::remove_file(&temporary);
        fail(e)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(folder: &Path) -> Vec<String> {
        let mut list: Vec<String> = fs::read_dir(folder)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        list.sort();
        list
    }

    #[test]
    fn a_new_file_is_written_and_no_temporary_file_is_left() {
        let work = tempfile::tempdir().unwrap();
        let path = work.path().join("parts.json");
        write_atomic(&path, "one\n", false).unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "one\n");
        assert_eq!(names(work.path()), vec!["parts.json"]);
    }

    #[test]
    fn an_existing_file_is_kept_unless_the_caller_orders_the_replacement() {
        let work = tempfile::tempdir().unwrap();
        let path = work.path().join("parts.json");
        write_atomic(&path, "one\n", false).unwrap();
        let refused = write_atomic(&path, "two\n", false).unwrap_err();
        assert_eq!(refused.code, ImportCode::OutputExists);
        assert_eq!(fs::read_to_string(&path).unwrap(), "one\n");
        assert_eq!(names(work.path()), vec!["parts.json"]);
        write_atomic(&path, "two\n", true).unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "two\n");
        assert_eq!(names(work.path()), vec!["parts.json"]);
    }

    #[test]
    fn a_missing_folder_is_a_refusal_not_a_panic() {
        let work = tempfile::tempdir().unwrap();
        let path = work.path().join("no-such-folder").join("parts.json");
        let e = write_atomic(&path, "x", false).unwrap_err();
        assert_eq!(e.code, ImportCode::WriteFailed);
        assert!(names(work.path()).is_empty());
    }
}
