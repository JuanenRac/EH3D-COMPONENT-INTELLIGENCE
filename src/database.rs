// SPDX-License-Identifier: GPL-3.0-or-later
//! The database: components, the observations of each field with their provenance, and the checks that find what is wrong.

use crate::config::Config;
use crate::error::CintError;
use crate::Code;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// How a value was obtained.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Method {
    /// Taken out of a document by a program.
    Extracted,
    /// Typed in by a person from a document.
    Transcribed,
    /// Concluded from other values; never read from a document.
    Inferred,
    /// Checked against the data sheet by a person.
    Verified,
    /// Measured on a real part.
    Measured,
}

/// One value of one field, with where it came from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Observation {
    /// The value, as text with its unit (`3.3 V`).
    pub value: String,
    /// How it was obtained.
    pub method: Method,
    /// The document or other source. Empty means nobody said: that is a finding.
    #[serde(default)]
    pub source: String,
    /// The page of the source.
    #[serde(default)]
    pub page: Option<String>,
    /// The table of the source.
    #[serde(default)]
    pub table: Option<String>,
    /// The revision of the source.
    #[serde(default)]
    pub revision: Option<String>,
    /// The date the source was consulted, as the person wrote it.
    #[serde(default)]
    pub consulted: Option<String>,
}

/// Whether a record was made by hand or imported.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OriginKind {
    /// Filled by hand.
    Manual,
    /// Imported from a file.
    Import,
}

/// Where a record came from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Origin {
    /// Manual or import.
    pub kind: OriginKind,
    /// The file or person it came from (required for an import).
    #[serde(default)]
    pub source: String,
}

/// How far a record is trusted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    /// Not reviewed completely.
    #[default]
    Draft,
    /// A person reviewed all of it.
    Verified,
}

/// One component.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Component {
    /// The manufacturer, as written on the data sheet.
    pub manufacturer: String,
    /// The exact part number.
    pub part_number: String,
    /// The confidence state.
    #[serde(default)]
    pub status: Status,
    /// The fields still to review.
    #[serde(default)]
    pub pending: Vec<String>,
    /// Where the record came from.
    #[serde(default)]
    pub origin: Option<Origin>,
    /// The licence under which the data may be redistributed, as written by the person who reviewed it.
    #[serde(default)]
    pub licence: Option<String>,
    /// The reference designators of the design that use this part (`R1`, `C3`), kept in order and without repeats by the importer.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub references: Vec<String>,
    /// The observations of each field, by field name.
    #[serde(default)]
    pub fields: BTreeMap<String, Vec<Observation>>,
}

impl Component {
    /// The identity: manufacturer and part number, trimmed and in lower case.
    pub fn identity(&self) -> (String, String) {
        (
            self.manufacturer.trim().to_lowercase(),
            self.part_number.trim().to_lowercase(),
        )
    }
}

/// Something wrong in the database.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    /// The code.
    pub code: Code,
    /// The manufacturer of the record.
    pub manufacturer: String,
    /// The part number of the record.
    pub part_number: String,
    /// The field it is about, when it is about one.
    pub field: Option<String>,
    /// What is wrong, in English.
    pub message: String,
}

/// The database.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Database {
    /// The components.
    pub components: Vec<Component>,
}

impl Database {
    /// Read a database from JSON, within the bounds of the configuration.
    pub fn from_json(text: &str, config: &Config) -> Result<Database, CintError> {
        if text.len() as u64 > config.max_database_bytes {
            return Err(CintError::TooLarge {
                what: "database bytes",
            });
        }
        let db: Database =
            serde_json::from_str(text).map_err(|e| CintError::BadJson(e.to_string()))?;
        if db.components.len() > config.max_components {
            return Err(CintError::TooLarge { what: "components" });
        }
        if db
            .components
            .iter()
            .any(|c| c.fields.values().any(|o| o.len() > config.max_observations))
        {
            return Err(CintError::TooLarge {
                what: "observations of a field",
            });
        }
        Ok(db)
    }

    /// The database as JSON (the order of the components is kept; the fields are sorted by name).
    pub fn to_json(&self) -> Result<String, CintError> {
        serde_json::to_string_pretty(self).map_err(|e| CintError::BadJson(e.to_string()))
    }

    /// The canonical form of the database for a file: components sorted by identity, references sorted, fields sorted by name, two-space
    /// indentation and one final line feed. The same content always gives the same text, so the file can be compared and kept under version control.
    pub fn to_canonical_json(&self) -> Result<String, CintError> {
        let mut sorted = self.clone();
        for c in &mut sorted.components {
            c.references.sort();
            for list in c.fields.values_mut() {
                list.sort_by(|a, b| (&a.value, &a.source).cmp(&(&b.value, &b.source)));
            }
        }
        sorted.components.sort_by(|a, b| {
            (a.identity(), &a.manufacturer, &a.part_number).cmp(&(
                b.identity(),
                &b.manufacturer,
                &b.part_number,
            ))
        });
        let mut text = sorted.to_json()?;
        text.push('\n');
        Ok(text)
    }

    /// Find a component by manufacturer and part number (letter case and surrounding spaces do not matter).
    pub fn find(&self, manufacturer: &str, part_number: &str) -> Option<&Component> {
        let wanted = (
            manufacturer.trim().to_lowercase(),
            part_number.trim().to_lowercase(),
        );
        self.components.iter().find(|c| c.identity() == wanted)
    }

    /// Every finding, in a fixed order (manufacturer, part number, code, field).
    pub fn check(&self, config: &Config) -> Vec<Finding> {
        let mut out = Vec::new();
        let mut seen = BTreeSet::new();
        for c in &self.components {
            let at = |code, field: Option<&str>, message: String| Finding {
                code,
                manufacturer: c.manufacturer.clone(),
                part_number: c.part_number.clone(),
                field: field.map(str::to_string),
                message,
            };
            if c.manufacturer.trim().is_empty() || c.part_number.trim().is_empty() {
                out.push(at(
                    Code::BadRecord,
                    None,
                    "the manufacturer and the part number must not be empty".into(),
                ));
            } else if !seen.insert(c.identity()) {
                out.push(at(
                    Code::DuplicateIdentity,
                    None,
                    "another record has the same manufacturer and part number".into(),
                ));
            }
            match &c.origin {
                None => out.push(at(
                    Code::NoOrigin,
                    None,
                    "the record does not say whether it was made by hand or imported".into(),
                )),
                Some(o) if o.kind == OriginKind::Import && o.source.trim().is_empty() => {
                    out.push(at(
                        Code::NoOrigin,
                        None,
                        "an import must say what it was imported from".into(),
                    ))
                }
                _ => {}
            }
            if c.licence.as_deref().is_none_or(|l| l.trim().is_empty()) {
                out.push(at(
                    Code::NoLicence,
                    None,
                    "the licence of the data is not recorded".into(),
                ));
            }
            let mut blocked = !c.pending.is_empty();
            for (name, observations) in &c.fields {
                if name.trim().is_empty() || observations.is_empty() {
                    out.push(at(
                        Code::BadRecord,
                        Some(name),
                        "a field needs a name and at least one observation".into(),
                    ));
                    blocked = true;
                    continue;
                }
                for o in observations {
                    if o.source.trim().is_empty() {
                        out.push(at(
                            Code::NoProvenance,
                            Some(name),
                            "an observation has no source".into(),
                        ));
                        blocked = true;
                    }
                    if config.inferred_blocks_verified && o.method == Method::Inferred {
                        blocked = true;
                    }
                }
                let values: BTreeSet<&str> = observations.iter().map(|o| o.value.trim()).collect();
                if values.len() > 1 {
                    let list: Vec<&str> = values.into_iter().collect();
                    out.push(at(
                        Code::Conflict,
                        Some(name),
                        format!("the sources disagree: {}", list.join(" | ")),
                    ));
                    blocked = true;
                }
            }
            if c.status == Status::Verified && blocked {
                out.push(at(
                    Code::VerifiedWithPending,
                    None,
                    "marked verified while something is pending, in conflict, inferred or without a source".into(),
                ));
            }
        }
        out.sort_by(|a, b| {
            (&a.manufacturer, &a.part_number, a.code, &a.field).cmp(&(
                &b.manufacturer,
                &b.part_number,
                b.code,
                &b.field,
            ))
        });
        out
    }

    /// The database as JSON for redistribution. With `require_licence_for_export` a record without a licence refuses the whole export, and the findings say which.
    pub fn export(&self, config: &Config) -> Result<String, Vec<Finding>> {
        if config.require_licence_for_export {
            let missing: Vec<Finding> = self
                .check(config)
                .into_iter()
                .filter(|f| f.code == Code::NoLicence)
                .collect();
            if !missing.is_empty() {
                return Err(missing);
            }
        }
        self.to_json().map_err(|_| Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn obs(value: &str, method: Method, source: &str) -> Observation {
        Observation {
            value: value.into(),
            method,
            source: source.into(),
            page: Some("3".into()),
            table: None,
            revision: Some("B".into()),
            consulted: Some("on the day it was typed".into()),
        }
    }

    fn component(part: &str) -> Component {
        let mut fields = BTreeMap::new();
        fields.insert(
            "vcc_max".to_string(),
            vec![obs("3.6 V", Method::Transcribed, "data sheet")],
        );
        Component {
            manufacturer: "Acme".into(),
            part_number: part.into(),
            status: Status::Draft,
            pending: vec![],
            origin: Some(Origin {
                kind: OriginKind::Manual,
                source: String::new(),
            }),
            licence: Some("own notes".into()),
            references: vec![],
            fields,
        }
    }

    fn codes(db: &Database) -> Vec<Code> {
        db.check(&Config::default())
            .iter()
            .map(|f| f.code)
            .collect()
    }

    #[test]
    fn a_clean_record_has_no_findings() {
        let db = Database {
            components: vec![component("X1")],
        };
        assert!(codes(&db).is_empty());
    }

    #[test]
    fn the_same_identity_twice_is_found_whatever_the_letter_case() {
        let mut b = component("x1");
        b.manufacturer = " ACME ".into();
        let db = Database {
            components: vec![component("X1"), b],
        };
        assert_eq!(codes(&db), vec![Code::DuplicateIdentity]);
        assert!(db.find("acme", "X1").is_some() && db.find("acme", "nope").is_none());
    }

    #[test]
    fn a_field_without_a_source_is_flagged() {
        let mut c = component("X1");
        c.fields.get_mut("vcc_max").unwrap()[0].source = "  ".into();
        assert_eq!(
            codes(&Database {
                components: vec![c]
            }),
            vec![Code::NoProvenance]
        );
    }

    #[test]
    fn two_sources_that_disagree_show_both_values() {
        let mut c = component("X1");
        c.fields
            .get_mut("vcc_max")
            .unwrap()
            .push(obs("5.5 V", Method::Extracted, "other sheet"));
        let db = Database {
            components: vec![c],
        };
        let f = db.check(&Config::default());
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].code, Code::Conflict);
        assert!(f[0].message.contains("3.6 V") && f[0].message.contains("5.5 V"));
    }

    #[test]
    fn a_record_with_pending_fields_is_never_verified() {
        let mut c = component("X1");
        c.status = Status::Verified;
        c.pending = vec!["footprint".into()];
        assert_eq!(
            codes(&Database {
                components: vec![c.clone()]
            }),
            vec![Code::VerifiedWithPending]
        );
        c.pending.clear();
        assert!(codes(&Database {
            components: vec![c.clone()]
        })
        .is_empty());
        // an inferred value blocks it by default, and not when the setting is off
        c.fields.insert(
            "rth".into(),
            vec![obs("40 K/W", Method::Inferred, "estimate")],
        );
        let db = Database {
            components: vec![c],
        };
        assert_eq!(codes(&db), vec![Code::VerifiedWithPending]);
        let relaxed = Config {
            inferred_blocks_verified: false,
            ..Config::default()
        };
        assert!(db.check(&relaxed).is_empty());
    }

    #[test]
    fn an_import_must_say_what_it_came_from_and_a_record_needs_an_origin() {
        let mut c = component("X1");
        c.origin = Some(Origin {
            kind: OriginKind::Import,
            source: String::new(),
        });
        let mut d = component("X2");
        d.origin = None;
        let db = Database {
            components: vec![c, d],
        };
        assert_eq!(codes(&db), vec![Code::NoOrigin, Code::NoOrigin]);
    }

    #[test]
    fn data_without_a_licence_is_not_exported() {
        let mut c = component("X1");
        c.licence = None;
        let db = Database {
            components: vec![component("X0"), c],
        };
        let refused = db.export(&Config::default()).unwrap_err();
        assert_eq!(refused.len(), 1);
        assert_eq!(refused[0].part_number, "X1");
        let open = Config {
            require_licence_for_export: false,
            ..Config::default()
        };
        assert!(db.export(&open).is_ok());
        let ok = Database {
            components: vec![component("X0")],
        };
        let text = ok.export(&Config::default()).unwrap();
        assert_eq!(Database::from_json(&text, &Config::default()).unwrap(), ok);
    }

    #[test]
    fn bad_records_and_bad_json_are_refused_without_panicking() {
        let mut c = component("");
        c.fields.insert("empty".into(), vec![]);
        let f = Database {
            components: vec![c],
        }
        .check(&Config::default());
        assert_eq!(f.iter().filter(|f| f.code == Code::BadRecord).count(), 2);
        let config = Config::default();
        for text in [
            "",
            "[]",
            "{\"components\":[{\"manufacturer\":1}]}",
            "{\"components\":3}",
        ] {
            assert!(matches!(
                Database::from_json(text, &config),
                Err(CintError::BadJson(_))
            ));
        }
        let small = Config {
            max_database_bytes: 3,
            ..Config::default()
        };
        assert!(matches!(
            Database::from_json("{\"components\":[]}", &small),
            Err(CintError::TooLarge { .. })
        ));
        let few = Config {
            max_observations: 1,
            ..Config::default()
        };
        let mut c = component("X1");
        c.fields
            .get_mut("vcc_max")
            .unwrap()
            .push(obs("3.6 V", Method::Measured, "bench"));
        let text = Database {
            components: vec![c],
        }
        .to_json()
        .unwrap();
        assert!(matches!(
            Database::from_json(&text, &few),
            Err(CintError::TooLarge { .. })
        ));
    }

    #[test]
    fn findings_come_in_a_fixed_order() {
        let mut a = component("B");
        a.licence = None;
        let mut b = component("A");
        b.licence = None;
        let db = Database {
            components: vec![a, b],
        };
        let parts: Vec<String> = db
            .check(&Config::default())
            .into_iter()
            .map(|f| f.part_number)
            .collect();
        assert_eq!(parts, vec!["A", "B"]);
    }
}
