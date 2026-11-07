// SPDX-License-Identifier: GPL-3.0-or-later
//! The import of a CSV list of parts into a database (the format is written in `docs/CSV_FORMAT.md`).
//!
//! The six columns are fixed: `reference`, `value`, `package`, `manufacturer`, `mpn` and `datasheet`. A line is one reference of the design; lines with
//! the same manufacturer and part number (letter case and surrounding spaces do not matter) become one component that lists all its references.
//! Nothing is guessed: when two lines give different values for the same part, both stay as observations and the check reports the conflict.
//! An import either succeeds completely or produces nothing: the problems are listed with their line.

use crate::config::Config;
use crate::csv::{self, CsvLimits};
use crate::database::{Component, Database, Method, Observation, Origin, OriginKind, Status};
use crate::issue::{ImportCode, Issue};
use std::collections::BTreeMap;

/// The columns every file must have.
pub const COLUMNS: [&str; 6] = [
    "reference",
    "value",
    "package",
    "manufacturer",
    "mpn",
    "datasheet",
];

/// The result of a successful import.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Imported {
    /// The database made from the file.
    pub database: Database,
    /// How many data rows the file had.
    pub rows: usize,
}

/// The bounds of the reader, taken from the settings.
pub fn limits(config: &Config) -> CsvLimits {
    CsvLimits {
        max_bytes: config.csv_max_bytes,
        max_rows: config.csv_max_rows,
        max_columns: config.csv_max_columns,
        max_field_bytes: config.csv_max_field_bytes,
    }
}

fn observation(value: &str, source: &str) -> Observation {
    Observation {
        value: value.to_string(),
        method: Method::Transcribed,
        source: source.to_string(),
        page: None,
        table: None,
        revision: None,
        consulted: None,
    }
}

/// Read the bytes of a CSV file into a database. `source` names the file in the origin and in the source of every observation.
pub fn import_csv(bytes: &[u8], source: &str, config: &Config) -> Result<Imported, Vec<Issue>> {
    let table = csv::parse(bytes, config.csv_separator, &limits(config)).map_err(|i| vec![i])?;
    let mut issues: Vec<Issue> = Vec::new();
    let mut column: BTreeMap<&str, usize> = BTreeMap::new();
    for (index, name) in table.header.iter().enumerate() {
        if !COLUMNS.contains(&name.as_str()) {
            issues.push(Issue::new(
                ImportCode::UnknownColumn,
                table.header_line,
                format!("the column `{name}` is not one of: {}", COLUMNS.join(", ")),
            ));
        } else if column.insert(name.as_str(), index).is_some() {
            issues.push(Issue::new(
                ImportCode::BadHeader,
                table.header_line,
                format!("the column `{name}` appears twice"),
            ));
        }
    }
    for name in COLUMNS {
        if !column.contains_key(name) {
            issues.push(Issue::new(
                ImportCode::BadHeader,
                table.header_line,
                format!("the header has no column `{name}`"),
            ));
        }
    }
    if !issues.is_empty() {
        issues.truncate(config.csv_max_issues);
        return Err(issues);
    }
    let at = |name: &str| column[name];
    let mut components: BTreeMap<(String, String), Component> = BTreeMap::new();
    let mut references: BTreeMap<String, usize> = BTreeMap::new();
    for row in &table.rows {
        if issues.len() >= config.csv_max_issues {
            break;
        }
        let get = |name: &str| row.fields[at(name)].trim();
        let (reference, manufacturer, mpn) = (get("reference"), get("manufacturer"), get("mpn"));
        let mut complete = true;
        for (name, text) in [
            ("reference", reference),
            ("manufacturer", manufacturer),
            ("mpn", mpn),
        ] {
            if text.is_empty() {
                complete = false;
                issues.push(Issue::new(
                    ImportCode::EmptyRequired,
                    row.line,
                    format!("the column `{name}` is empty"),
                ));
            }
        }
        if !reference.is_empty() {
            if let Some(first) = references.insert(reference.to_lowercase(), row.line) {
                complete = false;
                issues.push(Issue::new(
                    ImportCode::DuplicateReference,
                    row.line,
                    format!("the reference `{reference}` is already used on line {first}"),
                ));
            }
        }
        if !complete {
            continue;
        }
        let key = (manufacturer.to_lowercase(), mpn.to_lowercase());
        let component = components.entry(key).or_insert_with(|| Component {
            manufacturer: manufacturer.to_string(),
            part_number: mpn.to_string(),
            status: Status::Draft,
            pending: vec![],
            origin: Some(Origin {
                kind: OriginKind::Import,
                source: source.to_string(),
            }),
            licence: Some(config.import_licence.trim().to_string()).filter(|l| !l.is_empty()),
            references: vec![],
            fields: BTreeMap::new(),
        });
        if manufacturer < component.manufacturer.as_str() {
            component.manufacturer = manufacturer.to_string();
        }
        if mpn < component.part_number.as_str() {
            component.part_number = mpn.to_string();
        }
        component.references.push(reference.to_string());
        for name in ["value", "package", "datasheet"] {
            let text = get(name);
            if text.is_empty() {
                continue;
            }
            let list = component.fields.entry(name.to_string()).or_default();
            if !list.iter().any(|o| o.value.trim() == text) {
                list.push(observation(text, source));
            }
        }
    }
    if issues.is_empty() {
        if components.len() > config.max_components {
            issues.push(Issue::new(
                ImportCode::TooLarge,
                0,
                format!(
                    "the file makes more than {} components",
                    config.max_components
                ),
            ));
        }
        let too_many = components
            .values()
            .any(|c| c.fields.values().any(|o| o.len() > config.max_observations));
        if too_many {
            issues.push(Issue::new(
                ImportCode::TooLarge,
                0,
                format!(
                    "a field of one part has more than {} different values",
                    config.max_observations
                ),
            ));
        }
    }
    if !issues.is_empty() {
        issues.truncate(config.csv_max_issues);
        return Err(issues);
    }
    let mut database = Database::default();
    for (_, mut component) in components {
        component.references.sort();
        database.components.push(component);
    }
    Ok(Imported {
        database,
        rows: table.rows.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Code;

    const HEADER: &str = "reference,value,package,manufacturer,mpn,datasheet\n";

    fn run(body: &str) -> Result<Imported, Vec<Issue>> {
        import_csv(
            format!("{HEADER}{body}").as_bytes(),
            "bom.csv",
            &Config::default(),
        )
    }

    fn codes(result: Result<Imported, Vec<Issue>>) -> Vec<(ImportCode, usize)> {
        result
            .unwrap_err()
            .iter()
            .map(|i| (i.code, i.line))
            .collect()
    }

    const GOOD: &str = "R1,10k,0603,Yageo,RC0603FR-0710KL,https://example.com/a.pdf\n\
R2,10k,0603,YAGEO, rc0603fr-0710kl ,\n\
C1,\"100 nF, X7R\",0402,Murata,GRM155R71C104KA88D,\n";

    #[test]
    fn lines_with_the_same_part_become_one_component() {
        let imported = run(GOOD).unwrap();
        assert_eq!(imported.rows, 3);
        let list = &imported.database.components;
        assert_eq!(list.len(), 2);
        let (murata, yageo) = (&list[0], &list[1]);
        assert_eq!(
            (murata.manufacturer.as_str(), murata.part_number.as_str()),
            ("Murata", "GRM155R71C104KA88D")
        );
        assert_eq!(murata.references, vec!["C1"]);
        assert_eq!(murata.fields["value"][0].value, "100 nF, X7R");
        assert!(!murata.fields.contains_key("datasheet"));
        assert_eq!(yageo.references, vec!["R1", "R2"]);
        assert_eq!(yageo.fields["value"].len(), 1);
        assert_eq!(yageo.fields["package"].len(), 1);
        assert_eq!(
            yageo.fields["datasheet"][0].value,
            "https://example.com/a.pdf"
        );
        let o = &yageo.fields["value"][0];
        assert_eq!(
            (o.method, o.source.as_str()),
            (Method::Transcribed, "bom.csv")
        );
        let origin = yageo.origin.as_ref().unwrap();
        assert_eq!(
            (origin.kind, origin.source.as_str()),
            (OriginKind::Import, "bom.csv")
        );
    }

    #[test]
    fn imported_records_have_no_licence_unless_the_settings_give_one() {
        let db = run(GOOD).unwrap().database;
        assert!(db.components.iter().all(|c| c.licence.is_none()));
        let found: Vec<Code> = db
            .check(&Config::default())
            .iter()
            .map(|f| f.code)
            .collect();
        assert_eq!(found, vec![Code::NoLicence, Code::NoLicence]);
        let config = Config::parse("import_licence = own notes\n").unwrap();
        let db = import_csv(format!("{HEADER}{GOOD}").as_bytes(), "bom.csv", &config)
            .unwrap()
            .database;
        assert!(db.check(&config).is_empty());
        assert!(db.export(&config).is_ok());
    }

    #[test]
    fn two_values_for_one_part_are_kept_and_the_check_reports_the_conflict() {
        let db = run("R1,10k,0603,Yageo,P1,\nR2,12k,0603,Yageo,P1,\n")
            .unwrap()
            .database;
        assert_eq!(db.components.len(), 1);
        assert_eq!(db.components[0].fields["value"].len(), 2);
        assert_eq!(db.components[0].fields["package"].len(), 1);
        let found: Vec<Code> = db
            .check(&Config::default())
            .iter()
            .map(|f| f.code)
            .collect();
        assert!(found.contains(&Code::Conflict));
    }

    #[test]
    fn empty_required_values_and_repeated_references_are_listed_with_their_line() {
        let result = run("R1,1,a,M,P,\n,1,a,M,P,\nr1,1,a,M,Q,\nR3,1,a, ,,\n");
        assert_eq!(
            codes(result),
            vec![
                (ImportCode::EmptyRequired, 3),
                (ImportCode::DuplicateReference, 4),
                (ImportCode::EmptyRequired, 5),
                (ImportCode::EmptyRequired, 5),
            ]
        );
    }

    #[test]
    fn the_header_is_checked_before_any_row() {
        let bad = |text: &str| import_csv(text.as_bytes(), "x", &Config::default());
        let e = bad("reference,value,package,manufacturer,mpn\nR1,1,a,M,P\n").unwrap_err();
        assert_eq!((e[0].code, e[0].line), (ImportCode::BadHeader, 1));
        assert!(e[0].message.contains("datasheet"));
        let e = bad("Reference,Value,Package,Manufacturer,MPN,Datasheet,Extra\n").unwrap_err();
        assert_eq!((e[0].code, e[0].line), (ImportCode::UnknownColumn, 1));
        let e = bad("reference,value,value,package,manufacturer,mpn,datasheet\n").unwrap_err();
        assert_eq!(e[0].code, ImportCode::BadHeader);
        assert!(e[0].message.contains("twice"));
        assert!(bad("REFERENCE , Value,Package,Manufacturer,MPN,Datasheet\nR1,1,a,M,P,\n").is_ok());
        let e = bad("mpn,reference,datasheet,package,manufacturer,value\nP,R1,,a,M,1\n").unwrap();
        assert_eq!(e.database.components[0].fields["value"][0].value, "1");
        assert_eq!(e.database.components[0].part_number, "P");
    }

    #[test]
    fn the_number_of_reported_problems_is_limited() {
        let config = Config::parse("csv_max_issues = 2\n").unwrap();
        let text = format!("{HEADER},1,a,M,P,\n,1,a,M,P,\n,1,a,M,P,\n");
        let e = import_csv(text.as_bytes(), "x", &config).unwrap_err();
        assert_eq!(e.len(), 2);
    }

    #[test]
    fn the_limits_of_the_settings_apply_to_the_file_and_to_the_result() {
        let config = Config::parse("csv_max_rows = 2\n").unwrap();
        let text = format!("{HEADER}R1,1,a,M,P,\nR2,1,a,M,Q,\nR3,1,a,M,S,\n");
        let e = import_csv(text.as_bytes(), "x", &config).unwrap_err();
        assert_eq!((e[0].code, e[0].line), (ImportCode::TooManyRows, 4));
        let config = Config::parse("max_components = 1\n").unwrap();
        let text = format!("{HEADER}R1,1,a,M,P,\nR2,1,a,M,Q,\n");
        let e = import_csv(text.as_bytes(), "x", &config).unwrap_err();
        assert_eq!((e[0].code, e[0].line), (ImportCode::TooLarge, 0));
        let config = Config::parse("max_observations = 1\n").unwrap();
        let text = format!("{HEADER}R1,1,a,M,P,\nR2,2,a,M,P,\n");
        let e = import_csv(text.as_bytes(), "x", &config).unwrap_err();
        assert_eq!(e[0].code, ImportCode::TooLarge);
        let config = Config::parse("csv_separator = ;\n").unwrap();
        let text = "reference;value;package;manufacturer;mpn;datasheet\nR1;1,5;a;M;P;\n";
        let ok = import_csv(text.as_bytes(), "x", &config).unwrap();
        assert_eq!(ok.database.components[0].fields["value"][0].value, "1,5");
    }

    #[test]
    fn a_byte_order_mark_and_crlf_do_not_change_the_result() {
        let plain = run(GOOD).unwrap();
        let mut bytes = vec![0xEF, 0xBB, 0xBF];
        bytes.extend_from_slice(format!("{HEADER}{GOOD}").replace('\n', "\r\n").as_bytes());
        let other = import_csv(&bytes, "bom.csv", &Config::default()).unwrap();
        assert_eq!(plain, other);
    }

    #[test]
    fn the_canonical_json_does_not_depend_on_the_order_of_the_lines() {
        let a = run(GOOD).unwrap().database.to_canonical_json().unwrap();
        let reversed = "C1,\"100 nF, X7R\",0402,Murata,GRM155R71C104KA88D,\n\
R2,10k,0603,YAGEO, rc0603fr-0710kl ,\n\
R1,10k,0603,Yageo,RC0603FR-0710KL,https://example.com/a.pdf\n";
        let b = run(reversed).unwrap().database.to_canonical_json().unwrap();
        assert_eq!(a, b);
        assert!(a.ends_with("}\n"));
        assert!(a.contains("\"references\": [\n        \"R1\",\n        \"R2\"\n      ]"));
        assert!(a.find("Murata").unwrap() < a.find("YAGEO").unwrap());
        // of the spellings of one part the smallest one in byte order is written
        assert!(a.contains("\"manufacturer\": \"YAGEO\""));
        assert!(a.contains("\"part_number\": \"RC0603FR-0710KL\""));
    }
}
