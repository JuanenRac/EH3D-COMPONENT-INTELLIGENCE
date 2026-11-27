//! Tests of the behaviour of the library and of the command line: the limits tested at the exact edge, damaged and empty input, the stable
//! codes and the stable shape of the output, and properties that must hold for every input (round trip, idempotence, stable order).

use eh3d_component_intelligence::cli;
use eh3d_component_intelligence::config::Config;
use eh3d_component_intelligence::csv::{self, CsvLimits};
use eh3d_component_intelligence::database::{
    Component, Database, Method, Observation, Origin, OriginKind, Status,
};
use eh3d_component_intelligence::error::CintError;
use eh3d_component_intelligence::import::{import_csv, limits, COLUMNS};
use eh3d_component_intelligence::issue::{ImportCode, Issue};
use eh3d_component_intelligence::simulation::{
    damage, import_with_reexport, ExportTwin, Fault, SimRng,
};
use eh3d_component_intelligence::store::write_atomic;
use eh3d_component_intelligence::Code;
use std::collections::BTreeMap;
use std::fs;

const HEADER: &str = "reference,value,package,manufacturer,mpn,datasheet\n";

fn cfg() -> Config {
    Config::default()
}

fn csv_of(rows: &str) -> Vec<u8> {
    format!("{HEADER}{rows}").into_bytes()
}

fn obs(value: &str, method: Method, source: &str) -> Observation {
    Observation {
        value: value.into(),
        method,
        source: source.into(),
        page: None,
        table: None,
        revision: None,
        consulted: None,
    }
}

fn part(manufacturer: &str, number: &str) -> Component {
    let mut fields = BTreeMap::new();
    fields.insert(
        "vcc".to_string(),
        vec![obs("3.3 V", Method::Transcribed, "sheet")],
    );
    Component {
        manufacturer: manufacturer.into(),
        part_number: number.into(),
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

fn db(list: Vec<Component>) -> Database {
    Database { components: list }
}

fn codes_of(d: &Database, c: &Config) -> Vec<Code> {
    d.check(c).iter().map(|f| f.code).collect()
}

fn lim(max_bytes: usize, max_rows: usize, max_columns: usize, max_field_bytes: usize) -> CsvLimits {
    CsvLimits {
        max_bytes,
        max_rows,
        max_columns,
        max_field_bytes,
    }
}

fn wide() -> CsvLimits {
    lim(1 << 20, 1000, 16, 4096)
}

fn call(list: &[&str]) -> (i32, String, String) {
    let args: Vec<String> = list.iter().map(|s| s.to_string()).collect();
    let (mut out, mut err) = (Vec::new(), Vec::new());
    let code = cli::run(&args, &mut out, &mut err);
    (
        code,
        String::from_utf8(out).unwrap(),
        String::from_utf8(err).unwrap(),
    )
}

// ---------------------------------------------------------------- settings

#[test]
fn settings_ignore_the_spaces_around_keys_and_values() {
    let c = Config::parse("   max_components   =   9   \n").unwrap();
    assert_eq!(c.max_components, 9);
}

#[test]
fn settings_count_comments_and_blank_lines_in_the_line_number() {
    let e = Config::parse("# one\n\nnonsense\n").unwrap_err();
    assert!(matches!(e, CintError::ConfigSyntax { line: 3 }));
}

#[test]
fn settings_accept_the_six_spellings_of_a_flag() {
    for (word, want) in [
        ("true", true),
        ("on", true),
        ("yes", true),
        ("false", false),
        ("off", false),
        ("no", false),
    ] {
        let c = Config::parse(&format!("require_licence_for_export = {word}\n")).unwrap();
        assert_eq!(c.require_licence_for_export, want, "{word}");
    }
}

#[test]
fn settings_flags_are_case_sensitive() {
    for word in ["TRUE", "Yes", "1", "0", ""] {
        let e = Config::parse(&format!("inferred_blocks_verified = {word}\n")).unwrap_err();
        assert!(
            matches!(e, CintError::ConfigBadValue { line: 1, .. }),
            "{word}"
        );
    }
}

#[test]
fn settings_value_may_contain_an_equals_sign() {
    let c = Config::parse("import_licence = a=b\n").unwrap();
    assert_eq!(c.import_licence, "a=b");
}

#[test]
fn settings_last_line_of_a_key_wins() {
    let c = Config::parse("max_components = 3\nmax_components = 8\n").unwrap();
    assert_eq!(c.max_components, 8);
}

#[test]
fn settings_database_bytes_accepts_zero_and_refuses_negative() {
    assert_eq!(
        Config::parse("max_database_bytes = 0\n")
            .unwrap()
            .max_database_bytes,
        0
    );
    assert!(Config::parse("max_database_bytes = -1\n").is_err());
}

#[test]
fn settings_separator_accepts_tab_and_pipe_and_refuses_a_space() {
    assert_eq!(
        Config::parse("csv_separator = tab\n")
            .unwrap()
            .csv_separator,
        '\t'
    );
    assert_eq!(
        Config::parse("csv_separator = |\n").unwrap().csv_separator,
        '|'
    );
    assert!(Config::parse("csv_separator = \n").is_err());
}

#[test]
fn settings_columns_edge_is_six() {
    assert!(Config::parse("csv_max_columns = 5\n").is_err());
    assert_eq!(
        Config::parse("csv_max_columns = 6\n")
            .unwrap()
            .csv_max_columns,
        6
    );
}

#[test]
fn settings_default_file_has_one_trailing_line_feed_and_every_key() {
    let text = Config::default_file();
    assert!(text.ends_with('\n') && !text.ends_with("\n\n"));
    for key in [
        "max_database_bytes",
        "max_components",
        "max_observations",
        "require_licence_for_export",
        "inferred_blocks_verified",
        "csv_max_bytes",
        "csv_max_rows",
        "csv_max_columns",
        "csv_max_field_bytes",
        "csv_max_issues",
        "csv_separator",
        "import_licence",
    ] {
        assert!(text.contains(&format!("\n{key} =")), "{key}");
    }
}

#[test]
fn settings_default_file_is_stable_between_calls() {
    assert_eq!(Config::default_file(), Config::default_file());
}

#[test]
fn settings_errors_say_the_line_and_the_key() {
    let e = Config::parse("max_components = x\n").unwrap_err();
    assert_eq!(
        e.to_string(),
        "configuration line 1: bad value for `max_components`"
    );
    let e = Config::parse("\nzzz = 1\n").unwrap_err();
    assert_eq!(e.to_string(), "configuration line 2: unknown key `zzz`");
    assert_eq!(
        CintError::TooLarge { what: "components" }.to_string(),
        "limit exceeded: components"
    );
}

#[test]
fn settings_huge_numbers_are_refused_not_wrapped() {
    assert!(Config::parse("max_components = 99999999999999999999999999\n").is_err());
}

// ---------------------------------------------------------------- codes

#[test]
fn finding_codes_have_stable_text_and_declaration_order() {
    let all = [
        Code::DuplicateIdentity,
        Code::NoProvenance,
        Code::Conflict,
        Code::VerifiedWithPending,
        Code::NoOrigin,
        Code::NoLicence,
        Code::BadRecord,
    ];
    for (i, c) in all.iter().enumerate() {
        assert_eq!(c.as_str(), format!("PHOENIX-CINT-{:03}", i + 1));
    }
    let mut sorted = all;
    sorted.sort();
    assert_eq!(sorted, all);
}

#[test]
fn import_codes_are_ordered_like_their_numbers() {
    let mut all = [
        ImportCode::WriteFailed,
        ImportCode::TooLarge,
        ImportCode::BadQuote,
        ImportCode::TooManyRows,
    ];
    all.sort();
    let texts: Vec<&str> = all.iter().map(|c| c.as_str()).collect();
    assert_eq!(
        texts,
        vec![
            "EH3D-CINT-101",
            "EH3D-CINT-102",
            "EH3D-CINT-106",
            "EH3D-CINT-113"
        ]
    );
}

#[test]
fn an_issue_without_a_line_prints_no_line() {
    let i = Issue::new(ImportCode::BadHeader, 0, "no header");
    assert_eq!(i.to_string(), "EH3D-CINT-107: no header");
    assert!(!Issue::new(ImportCode::BadHeader, 7, "x")
        .to_string()
        .contains("EH3D-CINT-107:"));
}

// ---------------------------------------------------------------- database

#[test]
fn identity_is_trimmed_and_lower_case() {
    let c = part("  AcMe ", " X-1\t");
    assert_eq!(c.identity(), ("acme".to_string(), "x-1".to_string()));
}

#[test]
fn find_ignores_case_and_spaces_on_both_sides() {
    let d = db(vec![part(" ACME", "x1 ")]);
    assert!(d.find("acme", "X1").is_some());
    assert!(d.find("  Acme  ", "  X1").is_some());
    assert!(d.find("acme", "x2").is_none());
}

#[test]
fn an_empty_database_has_no_findings_and_a_stable_json() {
    let d = Database::default();
    assert!(d.check(&cfg()).is_empty());
    assert!(d.to_json().unwrap().contains("\"components\": []"));
    assert_eq!(
        d.to_canonical_json().unwrap(),
        format!("{}\n", d.to_json().unwrap())
    );
}

#[test]
fn json_without_the_components_key_is_refused() {
    assert!(matches!(
        Database::from_json("{}", &cfg()),
        Err(CintError::BadJson(_))
    ));
    assert!(matches!(
        Database::from_json("null", &cfg()),
        Err(CintError::BadJson(_))
    ));
}

#[test]
fn a_minimal_component_gets_the_conservative_defaults() {
    let text = r#"{"components":[{"manufacturer":"A","part_number":"B"}]}"#;
    let d = Database::from_json(text, &cfg()).unwrap();
    let c = &d.components[0];
    assert_eq!(c.status, Status::Draft);
    assert!(c.pending.is_empty() && c.origin.is_none() && c.licence.is_none());
    assert!(c.fields.is_empty() && c.references.is_empty());
}

#[test]
fn unknown_method_and_status_words_are_refused() {
    let bad_method = r#"{"components":[{"manufacturer":"A","part_number":"B","fields":{"v":[{"value":"1","method":"guessed","source":"s"}]}}]}"#;
    assert!(Database::from_json(bad_method, &cfg()).is_err());
    let bad_status = r#"{"components":[{"manufacturer":"A","part_number":"B","status":"maybe"}]}"#;
    assert!(Database::from_json(bad_status, &cfg()).is_err());
}

#[test]
fn methods_and_status_are_written_in_lower_case() {
    let mut c = part("A", "B");
    c.status = Status::Verified;
    let text = db(vec![c]).to_json().unwrap();
    assert!(text.contains("\"status\": \"verified\""));
    assert!(text.contains("\"method\": \"transcribed\""));
    assert!(text.contains("\"kind\": \"manual\""));
}

#[test]
fn references_are_left_out_of_the_json_when_empty() {
    let mut c = part("A", "B");
    assert!(!db(vec![c.clone()])
        .to_json()
        .unwrap()
        .contains("references"));
    c.references = vec!["R1".into()];
    assert!(db(vec![c]).to_json().unwrap().contains("references"));
}

#[test]
fn the_number_of_components_is_limited_at_the_exact_edge() {
    let two = db(vec![part("A", "1"), part("A", "2")]);
    let text = two.to_json().unwrap();
    let ok = Config {
        max_components: 2,
        ..cfg()
    };
    let tight = Config {
        max_components: 1,
        ..cfg()
    };
    assert!(Database::from_json(&text, &ok).is_ok());
    assert!(matches!(
        Database::from_json(&text, &tight),
        Err(CintError::TooLarge { what: "components" })
    ));
}

#[test]
fn the_observations_of_a_field_are_limited_at_the_exact_edge() {
    let mut c = part("A", "1");
    c.fields
        .get_mut("vcc")
        .unwrap()
        .push(obs("5 V", Method::Measured, "bench"));
    let text = db(vec![c]).to_json().unwrap();
    let ok = Config {
        max_observations: 2,
        ..cfg()
    };
    let tight = Config {
        max_observations: 1,
        ..cfg()
    };
    assert!(Database::from_json(&text, &ok).is_ok());
    assert!(Database::from_json(&text, &tight).is_err());
}

#[test]
fn the_size_of_the_database_is_limited_at_the_exact_byte() {
    let text = db(vec![part("A", "1")]).to_json().unwrap();
    let exact = Config {
        max_database_bytes: text.len() as u64,
        ..cfg()
    };
    let one_less = Config {
        max_database_bytes: text.len() as u64 - 1,
        ..cfg()
    };
    assert!(Database::from_json(&text, &exact).is_ok());
    assert!(matches!(
        Database::from_json(&text, &one_less),
        Err(CintError::TooLarge {
            what: "database bytes"
        })
    ));
}

#[test]
fn the_canonical_form_sorts_the_components_by_identity() {
    let d = db(vec![
        part("Zed", "1"),
        part("acme", "9"),
        part("ACME", "10"),
    ]);
    let canon = Database::from_json(&d.to_canonical_json().unwrap(), &cfg()).unwrap();
    let ids: Vec<(String, String)> = canon.components.iter().map(|c| c.identity()).collect();
    let mut sorted = ids.clone();
    sorted.sort();
    assert_eq!(ids, sorted);
    assert_eq!(ids[0].0, "acme");
}

#[test]
fn the_canonical_form_ends_in_one_line_feed() {
    let t = db(vec![part("A", "1")]).to_canonical_json().unwrap();
    assert!(t.ends_with("}\n") && !t.ends_with("\n\n"));
    assert!(!t.contains('\r'));
}

#[test]
fn the_canonical_form_is_idempotent() {
    let d = db(vec![part("B", "2"), part("A", "1")]);
    let once = d.to_canonical_json().unwrap();
    let again = Database::from_json(&once, &cfg())
        .unwrap()
        .to_canonical_json()
        .unwrap();
    assert_eq!(once, again);
}

#[test]
fn the_canonical_form_does_not_depend_on_the_order_of_the_input() {
    let (a, b, c) = (part("A", "1"), part("B", "2"), part("C", "3"));
    let one = db(vec![a.clone(), b.clone(), c.clone()])
        .to_canonical_json()
        .unwrap();
    let two = db(vec![c, a, b]).to_canonical_json().unwrap();
    assert_eq!(one, two);
}

#[test]
fn the_canonical_form_sorts_references_and_observations() {
    let mut c = part("A", "1");
    c.references = vec!["R9".into(), "C1".into(), "R10".into()];
    c.fields
        .get_mut("vcc")
        .unwrap()
        .insert(0, obs("9 V", Method::Measured, "z"));
    let t = db(vec![c]).to_canonical_json().unwrap();
    let back = Database::from_json(&t, &cfg()).unwrap();
    assert_eq!(back.components[0].references, vec!["C1", "R10", "R9"]);
    assert_eq!(back.components[0].fields["vcc"][0].value, "3.3 V");
}

#[test]
fn the_plain_json_keeps_the_order_of_the_components() {
    let d = db(vec![part("Z", "1"), part("A", "1")]);
    let back = Database::from_json(&d.to_json().unwrap(), &cfg()).unwrap();
    assert_eq!(back, d);
}

#[test]
fn an_unknown_key_at_any_level_is_refused_and_named() {
    for (key, text) in [
        ("extra", r#"{"components":[],"extra":1}"#),
        (
            "pendin",
            r#"{"components":[{"manufacturer":"A","part_number":"B","status":"verified","pendin":["footprint"]}]}"#,
        ),
        (
            "sauce",
            r#"{"components":[{"manufacturer":"A","part_number":"B","origin":{"kind":"manual","sauce":"x"}}]}"#,
        ),
        (
            "pgae",
            r#"{"components":[{"manufacturer":"A","part_number":"B","fields":{"v":[{"value":"1","method":"measured","pgae":"3"}]}}]}"#,
        ),
    ] {
        match Database::from_json(text, &cfg()) {
            Err(CintError::BadJson(message)) => assert!(
                message.contains("unknown field") && message.contains(key),
                "{message}"
            ),
            other => panic!("{key}: {other:?}"),
        }
    }
}

#[test]
fn an_empty_manufacturer_or_a_blank_part_number_is_a_bad_record_and_not_a_duplicate() {
    let d = db(vec![part("", "1"), part("", "1"), part("A", "   ")]);
    let found = codes_of(&d, &cfg());
    assert_eq!(found.iter().filter(|c| **c == Code::BadRecord).count(), 3);
    assert!(!found.contains(&Code::DuplicateIdentity));
}

#[test]
fn three_equal_identities_give_two_duplicate_findings() {
    let d = db(vec![part("A", "1"), part("a", "1"), part("A ", "1")]);
    let n = codes_of(&d, &cfg())
        .into_iter()
        .filter(|c| *c == Code::DuplicateIdentity)
        .count();
    assert_eq!(n, 2);
}

#[test]
fn a_licence_of_blanks_counts_as_no_licence() {
    let mut c = part("A", "1");
    c.licence = Some("   \t".into());
    assert_eq!(codes_of(&db(vec![c]), &cfg()), vec![Code::NoLicence]);
}

#[test]
fn a_field_with_no_name_is_a_bad_record_that_names_the_field() {
    let mut c = part("A", "1");
    c.fields
        .insert(" ".into(), vec![obs("1", Method::Measured, "s")]);
    let f = db(vec![c]).check(&cfg());
    assert_eq!(f.len(), 1);
    assert_eq!(f[0].code, Code::BadRecord);
    assert_eq!(f[0].field.as_deref(), Some(" "));
}

#[test]
fn values_that_differ_only_in_surrounding_spaces_do_not_conflict() {
    let mut c = part("A", "1");
    c.fields
        .get_mut("vcc")
        .unwrap()
        .push(obs("  3.3 V ", Method::Extracted, "other"));
    assert!(codes_of(&db(vec![c]), &cfg()).is_empty());
}

#[test]
fn a_conflict_lists_all_the_values_in_sorted_order() {
    let mut c = part("A", "1");
    let list = c.fields.get_mut("vcc").unwrap();
    list.push(obs("5 V", Method::Extracted, "b"));
    list.push(obs("1.8 V", Method::Extracted, "c"));
    let f = db(vec![c]).check(&cfg());
    assert_eq!(f.len(), 1);
    assert_eq!(f[0].message, "the sources disagree: 1.8 V | 3.3 V | 5 V");
}

#[test]
fn a_verified_record_with_nothing_wrong_has_no_findings() {
    let mut c = part("A", "1");
    c.status = Status::Verified;
    assert!(codes_of(&db(vec![c]), &cfg()).is_empty());
}

#[test]
fn a_verified_record_without_any_data_is_flagged_with_the_reason() {
    let mut empty = part("A", "1");
    empty.status = Status::Verified;
    empty.fields.clear();
    let mut blank = part("A", "2");
    blank.status = Status::Verified;
    blank.fields.get_mut("vcc").unwrap()[0].value = "  ".into();
    blank.pending = vec!["footprint".into()];
    let found = db(vec![empty.clone(), blank]).check(&cfg());
    assert_eq!(found.len(), 2);
    for f in &found {
        assert_eq!(f.code, Code::VerifiedWithPending);
        assert!(f.message.contains("holds no data"), "{}", f.message);
    }
    empty.status = Status::Draft;
    assert!(codes_of(&db(vec![empty]), &cfg()).is_empty());
}

#[test]
fn a_verified_record_without_any_data_is_not_exported_even_without_the_licence_rule() {
    let mut c = part("A", "1");
    c.status = Status::Verified;
    c.fields.clear();
    let open = Config {
        require_licence_for_export: false,
        ..cfg()
    };
    for config in [cfg(), open] {
        let refused = db(vec![part("A", "0"), c.clone()])
            .export(&config)
            .unwrap_err();
        assert_eq!(refused.len(), 1);
        assert_eq!(refused[0].part_number, "1");
        assert!(refused[0].message.contains("holds no data"));
    }
}

#[test]
fn a_draft_with_pending_fields_is_not_accused() {
    let mut c = part("A", "1");
    c.pending = vec!["footprint".into()];
    assert!(codes_of(&db(vec![c]), &cfg()).is_empty());
}

#[test]
fn a_verified_record_without_a_source_gets_both_findings_in_code_order() {
    let mut c = part("A", "1");
    c.status = Status::Verified;
    c.fields.get_mut("vcc").unwrap()[0].source.clear();
    assert_eq!(
        codes_of(&db(vec![c]), &cfg()),
        vec![Code::NoProvenance, Code::VerifiedWithPending]
    );
}

#[test]
fn a_measured_value_never_blocks_a_verified_record() {
    let mut c = part("A", "1");
    c.status = Status::Verified;
    c.fields.get_mut("vcc").unwrap()[0].method = Method::Measured;
    let strict = Config {
        inferred_blocks_verified: true,
        ..cfg()
    };
    assert!(codes_of(&db(vec![c]), &strict).is_empty());
}

#[test]
fn a_manual_origin_may_have_no_source_text() {
    let mut c = part("A", "1");
    c.origin = Some(Origin {
        kind: OriginKind::Manual,
        source: "  ".into(),
    });
    assert!(codes_of(&db(vec![c]), &cfg()).is_empty());
}

#[test]
fn the_export_without_the_rule_passes_a_record_with_no_licence() {
    let mut c = part("A", "1");
    c.licence = None;
    let open = Config {
        require_licence_for_export: false,
        ..cfg()
    };
    assert!(db(vec![c]).export(&open).is_ok());
}

#[test]
fn a_refused_export_lists_only_licence_findings_for_every_record() {
    let (mut a, mut b) = (part("A", "1"), part("B", "2"));
    a.licence = None;
    b.licence = Some("".into());
    b.origin = None;
    let refused = db(vec![a, b]).export(&cfg()).unwrap_err();
    assert_eq!(refused.len(), 2);
    assert!(refused.iter().all(|f| f.code == Code::NoLicence));
}

#[test]
fn an_export_is_the_plain_json_of_the_database() {
    let d = db(vec![part("A", "1")]);
    assert_eq!(d.export(&cfg()).unwrap(), d.to_json().unwrap());
}

#[test]
fn findings_are_ordered_by_manufacturer_then_part_then_code() {
    let (mut a, mut b, mut c) = (part("Zed", "1"), part("Acme", "2"), part("Acme", "1"));
    a.licence = None;
    b.licence = None;
    c.licence = None;
    let f = db(vec![a, b, c]).check(&cfg());
    let order: Vec<(String, String)> = f
        .iter()
        .map(|f| (f.manufacturer.clone(), f.part_number.clone()))
        .collect();
    assert_eq!(
        order,
        vec![
            ("Acme".to_string(), "1".to_string()),
            ("Acme".to_string(), "2".to_string()),
            ("Zed".to_string(), "1".to_string())
        ]
    );
}

#[test]
fn the_check_is_deterministic() {
    let mut a = part("A", "1");
    a.licence = None;
    a.origin = None;
    let d = db(vec![a, part("A", "1")]);
    assert_eq!(d.check(&cfg()), d.check(&cfg()));
}

// ---------------------------------------------------------------- csv reader

#[test]
fn an_empty_file_has_no_header() {
    let e = csv::parse(b"", ',', &wide()).unwrap_err();
    assert_eq!(e.code, ImportCode::BadHeader);
}

#[test]
fn a_file_of_blank_lines_has_no_header() {
    let e = csv::parse(b"\n\n\r\n", ',', &wide()).unwrap_err();
    assert_eq!(e.code, ImportCode::BadHeader);
}

#[test]
fn a_header_alone_gives_a_table_without_rows() {
    let t = csv::parse(b"a,b\n", ',', &wide()).unwrap();
    assert_eq!(t.header, vec!["a", "b"]);
    assert!(t.rows.is_empty());
}

#[test]
fn the_byte_order_mark_is_ignored() {
    let mut bytes = vec![0xEF, 0xBB, 0xBF];
    bytes.extend_from_slice(b"a,b\n1,2\n");
    let t = csv::parse(&bytes, ',', &wide()).unwrap();
    assert_eq!(t.header[0], "a");
    assert_eq!(t.rows[0].fields, vec!["1", "2"]);
}

#[test]
fn crlf_and_lf_give_the_same_table() {
    let lf = csv::parse(b"a,b\n1,2\n3,4\n", ',', &wide()).unwrap();
    let crlf = csv::parse(b"a,b\r\n1,2\r\n3,4\r\n", ',', &wide()).unwrap();
    assert_eq!(lf, crlf);
}

#[test]
fn a_file_without_a_final_line_break_is_read() {
    let t = csv::parse(b"a,b\n1,2", ',', &wide()).unwrap();
    assert_eq!(t.rows.len(), 1);
}

#[test]
fn a_lone_carriage_return_is_refused() {
    let e = csv::parse(b"a,b\r1,2\n", ',', &wide()).unwrap_err();
    assert_eq!(e.code, ImportCode::BadQuote);
}

#[test]
fn a_quoted_field_may_hold_the_separator_and_line_breaks_and_lines_are_counted() {
    let t = csv::parse(b"h1,h2\n\"a,\nb\",c\nx,y\n", ',', &wide()).unwrap();
    assert_eq!(t.rows[0].fields, vec!["a,\nb", "c"]);
    assert_eq!(t.rows[0].line, 2);
    assert_eq!(t.rows[1].line, 4);
}

#[test]
fn a_doubled_quotation_mark_is_one_quotation_mark() {
    let t = csv::parse(b"h\n\"say \"\"hi\"\"\"\n", ',', &wide()).unwrap();
    assert_eq!(t.rows[0].fields[0], "say \"hi\"");
}

#[test]
fn a_quotation_mark_in_the_middle_of_a_field_is_refused() {
    let e = csv::parse(b"h1,h2\nab\"c,d\n", ',', &wide()).unwrap_err();
    assert_eq!((e.code, e.line), (ImportCode::BadQuote, 2));
}

#[test]
fn text_after_a_closing_quotation_mark_is_refused() {
    let e = csv::parse(b"h1,h2\n\"a\"b,c\n", ',', &wide()).unwrap_err();
    assert_eq!(e.code, ImportCode::BadQuote);
}

#[test]
fn an_unclosed_quotation_mark_names_the_line_where_it_opened() {
    let e = csv::parse(b"h\nok\n\"never closed\nmore\n", ',', &wide()).unwrap_err();
    assert_eq!((e.code, e.line), (ImportCode::BadQuote, 3));
}

#[test]
fn invalid_utf8_names_the_line_of_the_first_bad_byte() {
    let e = csv::parse(b"h\nok\n\xFF\n", ',', &wide()).unwrap_err();
    assert_eq!((e.code, e.line), (ImportCode::BadEncoding, 3));
}

#[test]
fn the_names_of_the_header_are_trimmed_and_in_lower_case_but_the_data_is_not() {
    let t = csv::parse(b" Ref , VAL \n  x , Y \n", ',', &wide()).unwrap();
    assert_eq!(t.header, vec!["ref", "val"]);
    assert_eq!(t.rows[0].fields, vec!["  x ", " Y "]);
}

#[test]
fn a_row_with_another_width_names_both_widths() {
    let e = csv::parse(b"a,b,c\n1,2\n", ',', &wide()).unwrap_err();
    assert_eq!((e.code, e.line), (ImportCode::RowWidth, 2));
    assert!(e.message.contains('2') && e.message.contains('3'));
}

#[test]
fn the_rows_are_limited_at_the_exact_edge() {
    let l = lim(1 << 20, 2, 16, 4096);
    assert!(csv::parse(b"a\n1\n2\n", ',', &l).is_ok());
    let e = csv::parse(b"a\n1\n2\n3\n", ',', &l).unwrap_err();
    assert_eq!((e.code, e.line), (ImportCode::TooManyRows, 4));
}

#[test]
fn the_columns_are_limited_at_the_exact_edge() {
    let l = lim(1 << 20, 10, 3, 4096);
    assert!(csv::parse(b"a,b,c\n1,2,3\n", ',', &l).is_ok());
    let e = csv::parse(b"a,b,c,d\n", ',', &l).unwrap_err();
    assert_eq!(e.code, ImportCode::TooManyColumns);
}

#[test]
fn the_size_of_a_field_is_limited_at_the_exact_byte() {
    let l = lim(1 << 20, 10, 16, 4);
    assert!(csv::parse(b"h\nabcd\n", ',', &l).is_ok());
    let e = csv::parse(b"h\nabcde\n", ',', &l).unwrap_err();
    assert_eq!((e.code, e.line), (ImportCode::FieldTooLong, 2));
    let e = csv::parse(b"h\n\"abcde\"\n", ',', &l).unwrap_err();
    assert_eq!(e.code, ImportCode::FieldTooLong);
}

#[test]
fn the_size_of_the_file_is_limited_at_the_exact_byte() {
    let bytes = b"a\n1\n";
    let l = lim(bytes.len(), 10, 16, 4096);
    assert!(csv::parse(bytes, ',', &l).is_ok());
    let l = lim(bytes.len() - 1, 10, 16, 4096);
    let e = csv::parse(bytes, ',', &l).unwrap_err();
    assert_eq!((e.code, e.line), (ImportCode::TooLarge, 0));
}

#[test]
fn other_separators_are_read() {
    let t = csv::parse(b"a;b\n1;2\n", ';', &wide()).unwrap();
    assert_eq!(t.rows[0].fields, vec!["1", "2"]);
    let t = csv::parse(b"a\tb\n1\t2\n", '\t', &wide()).unwrap();
    assert_eq!(t.header, vec!["a", "b"]);
    let t = csv::parse(b"a;b\n1,5;2\n", ';', &wide()).unwrap();
    assert_eq!(t.rows[0].fields[0], "1,5");
}

#[test]
fn blank_lines_are_skipped_but_the_line_numbers_go_on() {
    let t = csv::parse(b"a\n\n1\n\n\n2\n", ',', &wide()).unwrap();
    assert_eq!(t.rows.len(), 2);
    assert_eq!((t.rows[0].line, t.rows[1].line), (3, 6));
}

#[test]
fn a_quoted_empty_field_is_a_row_and_an_unquoted_one_is_not() {
    let t = csv::parse(b"h\n\"\"\n\n", ',', &wide()).unwrap();
    assert_eq!(t.rows.len(), 1);
    assert_eq!(t.rows[0].fields, vec![""]);
}

#[test]
fn a_large_file_of_many_rows_is_read_in_order() {
    let mut text = String::from("n\n");
    for i in 0..20_000 {
        text.push_str(&format!("{i}\n"));
    }
    let t = csv::parse(text.as_bytes(), ',', &lim(1 << 24, 100_000, 16, 4096)).unwrap();
    assert_eq!(t.rows.len(), 20_000);
    assert_eq!(t.rows[19_999].fields[0], "19999");
    assert_eq!(t.rows[19_999].line, 20_001);
}

#[test]
fn random_bytes_never_panic_the_reader() {
    let mut rng = SimRng::new(7);
    for _ in 0..300 {
        let n = rng.below(60) as usize;
        let bytes: Vec<u8> = (0..n).map(|_| rng.below(256) as u8).collect();
        let _ = csv::parse(&bytes, ',', &wide());
    }
}

// ---------------------------------------------------------------- import

#[test]
fn lines_of_the_same_part_become_one_component_with_sorted_references() {
    let bytes =
        csv_of("R3,10k,0603,Yageo,RC0603,\nR1,10k,0603,YAGEO ,rc0603,\nC1,1u,0402,Murata,GRM,\n");
    let got = import_csv(&bytes, "p.csv", &cfg()).unwrap();
    assert_eq!(got.rows, 3);
    assert_eq!(got.database.components.len(), 2);
    let y = got.database.find("yageo", "rc0603").unwrap();
    assert_eq!(y.references, vec!["R1", "R3"]);
    assert_eq!(y.fields["value"].len(), 1);
}

#[test]
fn the_columns_may_come_in_any_order_and_in_any_letter_case() {
    let bytes = b"MPN,Manufacturer,Reference\nP1,Acme,R1\n".to_vec();
    let e = import_csv(&bytes, "p.csv", &cfg()).unwrap_err();
    assert!(e.iter().all(|i| i.code == ImportCode::BadHeader));
    let full =
        b"DataSheet,PACKAGE,Value,MPN,Manufacturer,Reference\n,0402,1k,P1,Acme,R1\n".to_vec();
    assert!(import_csv(&full, "p.csv", &cfg()).is_ok());
}

#[test]
fn a_missing_column_is_named() {
    let e = import_csv(b"reference,value\nR1,1\n", "p.csv", &cfg()).unwrap_err();
    assert_eq!(e.len(), 4);
    assert!(e.iter().any(|i| i.message.contains("`mpn`")));
}

#[test]
fn an_unknown_column_is_refused_with_its_code() {
    let bytes = format!("{},colour\nR1,1,0402,A,P,,red\n", HEADER.trim_end()).into_bytes();
    let e = import_csv(&bytes, "p.csv", &cfg()).unwrap_err();
    assert_eq!(e.len(), 1);
    assert_eq!(e[0].code, ImportCode::UnknownColumn);
    assert_eq!(e[0].line, 1);
}

#[test]
fn a_repeated_column_is_refused() {
    let bytes = format!("{},mpn\nR1,1,0402,A,P,,Q\n", HEADER.trim_end()).into_bytes();
    let e = import_csv(&bytes, "p.csv", &cfg()).unwrap_err();
    assert!(e
        .iter()
        .any(|i| i.code == ImportCode::BadHeader && i.message.contains("twice")));
}

#[test]
fn a_row_with_three_empty_required_columns_gives_three_issues() {
    let e = import_csv(&csv_of(" , ,0402, , ,\n"), "p.csv", &cfg()).unwrap_err();
    assert_eq!(e.len(), 3);
    assert!(e
        .iter()
        .all(|i| i.code == ImportCode::EmptyRequired && i.line == 2));
}

#[test]
fn the_optional_columns_may_be_empty() {
    let got = import_csv(&csv_of("R1,,,Acme,P1,\n"), "p.csv", &cfg()).unwrap();
    assert!(got.database.components[0].fields.is_empty());
}

#[test]
fn a_repeated_reference_is_refused_whatever_its_letter_case() {
    let e = import_csv(&csv_of("R1,1,0402,A,P,\nr1,2,0402,A,Q,\n"), "p.csv", &cfg()).unwrap_err();
    assert_eq!(e.len(), 1);
    assert_eq!((e[0].code, e[0].line), (ImportCode::DuplicateReference, 3));
    assert!(e[0].message.contains("line 2"));
}

#[test]
fn the_list_of_issues_is_cut_at_the_setting() {
    let mut rows = String::new();
    for i in 0..30 {
        rows.push_str(&format!("R{i},1,0402,,,\n"));
    }
    let few = Config {
        csv_max_issues: 5,
        ..cfg()
    };
    let e = import_csv(&csv_of(&rows), "p.csv", &few).unwrap_err();
    assert!(e.len() <= 5 && !e.is_empty());
}

#[test]
fn one_bad_row_makes_the_whole_import_fail() {
    let e = import_csv(&csv_of("R1,1,0402,A,P,\nR2,1,0402,,Q,\n"), "p.csv", &cfg());
    assert!(e.is_err());
}

#[test]
fn the_licence_of_the_settings_is_written_on_every_record() {
    let with = Config {
        import_licence: "  CC0  ".into(),
        ..cfg()
    };
    let got = import_csv(&csv_of("R1,1,0402,A,P,\n"), "p.csv", &with).unwrap();
    assert_eq!(got.database.components[0].licence.as_deref(), Some("CC0"));
    let none = import_csv(&csv_of("R1,1,0402,A,P,\n"), "p.csv", &cfg()).unwrap();
    assert_eq!(none.database.components[0].licence, None);
}

#[test]
fn the_origin_and_the_sources_name_the_file() {
    let got = import_csv(&csv_of("R1,1k,0402,A,P,http://x\n"), "list.csv", &cfg()).unwrap();
    let c = &got.database.components[0];
    let o = c.origin.as_ref().unwrap();
    assert_eq!(
        (o.kind, o.source.as_str()),
        (OriginKind::Import, "list.csv")
    );
    assert!(c
        .fields
        .values()
        .flatten()
        .all(|ob| ob.source == "list.csv"));
    assert!(c
        .fields
        .values()
        .flatten()
        .all(|ob| ob.method == Method::Transcribed));
    assert_eq!(c.status, Status::Draft);
}

#[test]
fn two_different_values_for_one_part_stay_and_the_check_reports_the_conflict() {
    let got = import_csv(
        &csv_of("R1,1k,0402,A,P,\nR2,2k,0402,A,P,\n"),
        "p.csv",
        &cfg(),
    )
    .unwrap();
    assert_eq!(got.database.components[0].fields["value"].len(), 2);
    let found = codes_of(&got.database, &cfg());
    assert!(found.contains(&Code::Conflict));
}

#[test]
fn an_import_without_a_licence_has_only_the_licence_finding() {
    let got = import_csv(&csv_of("R1,1k,0402,A,P,\n"), "p.csv", &cfg()).unwrap();
    assert_eq!(codes_of(&got.database, &cfg()), vec![Code::NoLicence]);
}

#[test]
fn an_import_with_a_licence_has_no_findings() {
    let with = Config {
        import_licence: "CC0".into(),
        ..cfg()
    };
    let got = import_csv(
        &csv_of("R1,1k,0402,A,P,\nR2,1k,0402,A,P,\n"),
        "p.csv",
        &with,
    )
    .unwrap();
    assert!(codes_of(&got.database, &with).is_empty());
}

#[test]
fn the_spelling_with_the_smallest_bytes_is_kept_whatever_the_order() {
    let a = import_csv(
        &csv_of("R1,1,0402,Yageo,P1,\nR2,1,0402,yageo,p1,\n"),
        "p.csv",
        &cfg(),
    )
    .unwrap();
    let b = import_csv(
        &csv_of("R2,1,0402,yageo,p1,\nR1,1,0402,Yageo,P1,\n"),
        "p.csv",
        &cfg(),
    )
    .unwrap();
    assert_eq!(a.database.components[0].manufacturer, "Yageo");
    assert_eq!(a.database.components[0].part_number, "P1");
    assert_eq!(a.database.components, b.database.components);
}

#[test]
fn too_many_components_are_refused_at_the_exact_edge() {
    let rows = "R1,1,0402,A,P1,\nR2,1,0402,A,P2,\n";
    let ok = Config {
        max_components: 2,
        ..cfg()
    };
    let tight = Config {
        max_components: 1,
        ..cfg()
    };
    assert!(import_csv(&csv_of(rows), "p.csv", &ok).is_ok());
    let e = import_csv(&csv_of(rows), "p.csv", &tight).unwrap_err();
    assert_eq!(e[0].code, ImportCode::TooLarge);
}

#[test]
fn too_many_values_for_one_field_are_refused_at_the_exact_edge() {
    let rows = "R1,1k,0402,A,P,\nR2,2k,0402,A,P,\n";
    let ok = Config {
        max_observations: 2,
        ..cfg()
    };
    let tight = Config {
        max_observations: 1,
        ..cfg()
    };
    assert!(import_csv(&csv_of(rows), "p.csv", &ok).is_ok());
    assert!(import_csv(&csv_of(rows), "p.csv", &tight).is_err());
}

#[test]
fn a_csv_problem_of_the_reader_comes_as_one_issue() {
    let e = import_csv(b"", "p.csv", &cfg()).unwrap_err();
    assert_eq!(e.len(), 1);
    assert_eq!(e[0].code, ImportCode::BadHeader);
}

#[test]
fn the_import_is_deterministic_and_survives_the_canonical_round_trip() {
    let rows = "R2,1k,0402,B,Q,\nR1,2k,0603,A,P,s\nR3,2k,0603,a,p,\n";
    let one = import_csv(&csv_of(rows), "p.csv", &cfg()).unwrap();
    let two = import_csv(&csv_of(rows), "p.csv", &cfg()).unwrap();
    assert_eq!(one, two);
    let text = one.database.to_canonical_json().unwrap();
    let back = Database::from_json(&text, &cfg()).unwrap();
    assert_eq!(back.to_canonical_json().unwrap(), text);
}

#[test]
fn the_limits_are_taken_from_the_settings() {
    let c = Config {
        csv_max_bytes: 11,
        csv_max_rows: 12,
        csv_max_columns: 13,
        csv_max_field_bytes: 14,
        ..cfg()
    };
    assert_eq!(limits(&c), lim(11, 12, 13, 14));
    assert_eq!(COLUMNS.len(), 6);
}

#[test]
fn a_semicolon_file_is_imported_with_the_setting() {
    let semi = Config {
        csv_separator: ';',
        ..cfg()
    };
    let text = "reference;value;package;manufacturer;mpn;datasheet\nR1;1k;0402;A;P;\n";
    assert!(import_csv(text.as_bytes(), "p.csv", &semi).is_ok());
    assert!(import_csv(text.as_bytes(), "p.csv", &cfg()).is_err());
}

// ---------------------------------------------------------------- writing

#[test]
fn a_path_without_a_file_name_is_a_refusal() {
    let e = write_atomic(std::path::Path::new(".."), "x", true).unwrap_err();
    assert_eq!(e.code, ImportCode::WriteFailed);
}

#[test]
fn replacing_a_file_that_does_not_exist_creates_it() {
    let work = tempfile::tempdir().unwrap();
    let path = work.path().join("new.json");
    write_atomic(&path, "data\n", true).unwrap();
    assert_eq!(fs::read_to_string(&path).unwrap(), "data\n");
}

#[test]
fn an_empty_text_is_written_as_an_empty_file() {
    let work = tempfile::tempdir().unwrap();
    let path = work.path().join("e.json");
    write_atomic(&path, "", false).unwrap();
    assert_eq!(fs::metadata(&path).unwrap().len(), 0);
}

#[test]
fn the_refusal_of_an_existing_file_names_the_file_and_the_option() {
    let work = tempfile::tempdir().unwrap();
    let path = work.path().join("a.json");
    fs::write(&path, "keep").unwrap();
    let e = write_atomic(&path, "new", false).unwrap_err();
    assert!(e.message.contains("a.json") && e.message.contains("--overwrite"));
    assert_eq!(fs::read_to_string(&path).unwrap(), "keep");
}

// ---------------------------------------------------------------- simulation

#[test]
fn the_generator_gives_the_same_numbers_for_the_same_seed() {
    let (mut a, mut b) = (SimRng::new(42), SimRng::new(42));
    let one: Vec<u64> = (0..16).map(|_| a.next_u64()).collect();
    let two: Vec<u64> = (0..16).map(|_| b.next_u64()).collect();
    assert_eq!(one, two);
    let mut c = SimRng::new(43);
    assert_ne!(one[0], c.next_u64());
}

#[test]
fn below_stays_below_the_bound_and_one_gives_zero() {
    let mut r = SimRng::new(1);
    for bound in [1u64, 2, 7, 1000] {
        for _ in 0..200 {
            assert!(r.below(bound) < bound);
        }
    }
    assert_eq!(r.below(1), 0);
}

#[test]
fn the_twin_repeats_its_export_for_a_seed() {
    let one = ExportTwin::new(9).export(25);
    let two = ExportTwin::new(9).export(25);
    assert_eq!(one, two);
    assert_ne!(one, ExportTwin::new(10).export(25));
    assert_eq!(one.rows.len(), 25);
}

#[test]
fn the_twin_export_has_the_six_columns_and_unique_references() {
    let e = ExportTwin::new(3).export(40);
    assert_eq!(e.header, COLUMNS.to_vec());
    assert!(e.rows.iter().all(|r| r.len() == 6));
    let mut refs: Vec<&String> = e.rows.iter().map(|r| &r[0]).collect();
    refs.sort();
    refs.dedup();
    assert_eq!(refs.len(), 40);
    assert!(e.parts() <= 40 && e.parts() >= 1);
}

#[test]
fn a_twin_export_is_always_importable_and_keeps_every_row() {
    for seed in 0..12 {
        let e = ExportTwin::new(seed).export(60);
        let got = import_csv(&e.to_csv(','), "x.csv", &cfg()).unwrap();
        assert_eq!(got.rows, 60, "{seed}");
        assert_eq!(got.database.components.len(), e.parts(), "{seed}");
        let refs: usize = got
            .database
            .components
            .iter()
            .map(|c| c.references.len())
            .sum();
        assert_eq!(refs, 60, "{seed}");
    }
}

#[test]
fn the_twin_export_survives_every_separator() {
    let e = ExportTwin::new(5).export(30);
    for sep in [',', ';', '\t', '|'] {
        let c = Config {
            csv_separator: sep,
            ..cfg()
        };
        assert!(import_csv(&e.to_csv(sep), "x.csv", &c).is_ok(), "{sep:?}");
    }
}

#[test]
fn an_export_with_no_rows_has_no_parts_and_a_header_only_file() {
    let e = ExportTwin::new(1).export(0);
    assert_eq!(e.parts(), 0);
    let t = csv::parse(&e.to_csv(','), ',', &wide()).unwrap();
    assert!(t.rows.is_empty());
}

#[test]
fn each_kind_of_damage_gives_its_own_code() {
    let e = ExportTwin::new(2).export(20);
    for (fault, want) in [
        (Fault::InvalidUtf8, ImportCode::BadEncoding),
        (Fault::MissingColumn, ImportCode::BadHeader),
        (Fault::ExtraColumn, ImportCode::UnknownColumn),
        (Fault::RaggedRow, ImportCode::RowWidth),
        (Fault::DuplicateReference, ImportCode::DuplicateReference),
        (Fault::EmptyPartNumber, ImportCode::EmptyRequired),
        (Fault::UnclosedQuote, ImportCode::BadQuote),
    ] {
        let bytes = damage(&e, fault, ',').unwrap();
        let issues = import_csv(&bytes, "x.csv", &cfg()).unwrap_err();
        assert_eq!(issues[0].code, want, "{fault:?}");
    }
}

#[test]
fn damage_needs_at_least_two_rows() {
    for lines in [0usize, 1] {
        let e = ExportTwin::new(2).export(lines);
        assert!(damage(&e, Fault::RaggedRow, ',').is_none());
        assert!(damage(&e, Fault::InvalidUtf8, ',').is_none());
    }
}

#[test]
fn damage_does_not_change_the_original_export() {
    let e = ExportTwin::new(4).export(10);
    let before = e.clone();
    let _ = damage(&e, Fault::EmptyPartNumber, ',');
    assert_eq!(e, before);
}

#[test]
fn damage_is_found_with_another_separator_too() {
    let e = ExportTwin::new(6).export(15);
    let semi = Config {
        csv_separator: ';',
        ..cfg()
    };
    let bytes = damage(&e, Fault::RaggedRow, ';').unwrap();
    let issues = import_csv(&bytes, "x.csv", &semi).unwrap_err();
    assert_eq!(issues[0].code, ImportCode::RowWidth);
}

#[test]
fn a_damaged_file_is_recovered_by_a_new_export_within_the_attempts() {
    let mut twin = ExportTwin::new(8);
    let first = damage(&twin.export(12), Fault::RaggedRow, ',').unwrap();
    let (result, read) = import_with_reexport(&first, &mut twin, 12, 3, &cfg());
    assert!(result.is_ok());
    assert_eq!(read, 2);
}

#[test]
fn with_one_attempt_the_damage_stays_and_nothing_is_asked_again() {
    let mut twin = ExportTwin::new(8);
    let first = damage(&twin.export(12), Fault::InvalidUtf8, ',').unwrap();
    let (result, read) = import_with_reexport(&first, &mut twin, 12, 1, &cfg());
    assert!(result.is_err());
    assert_eq!(read, 1);
}

#[test]
fn a_good_first_file_is_read_once() {
    let mut twin = ExportTwin::new(8);
    let first = twin.export(12).to_csv(',');
    let (result, read) = import_with_reexport(&first, &mut twin, 12, 5, &cfg());
    assert!(result.is_ok());
    assert_eq!(read, 1);
}

// ---------------------------------------------------------------- command line

const GOOD: &str = r#"{"components":[{"manufacturer":"Acme","part_number":"X1","references":["R1","R2"],"status":"verified","origin":{"kind":"manual"},
    "licence":"own notes","fields":{"vcc":[{"value":"3.3 V","method":"transcribed","source":"sheet"}]}}]}"#;

fn write_in(dir: &tempfile::TempDir, name: &str, text: &str) -> String {
    let path = dir.path().join(name);
    fs::write(&path, text).unwrap();
    path.to_str().unwrap().to_string()
}

#[test]
fn no_arguments_print_the_usage_and_exit_2() {
    let (code, out, err) = call(&[]);
    assert_eq!(code, 2);
    assert!(out.is_empty());
    for word in [
        "check",
        "show",
        "export",
        "list",
        "check-csv",
        "import",
        "Exit codes",
    ] {
        assert!(err.contains(word), "{word}");
    }
}

#[test]
fn a_command_with_the_wrong_number_of_arguments_exits_2() {
    let work = tempfile::tempdir().unwrap();
    let f = write_in(&work, "d.json", GOOD);
    assert_eq!(call(&["list", &f, "extra"]).0, 2);
    assert_eq!(call(&["show", &f, "acme"]).0, 2);
    assert_eq!(call(&["import", &f]).0, 2);
    assert_eq!(call(&["import", &f, "a", "b", "c"]).0, 2);
}

#[test]
fn list_prints_one_line_per_component_and_the_count() {
    let work = tempfile::tempdir().unwrap();
    let f = write_in(&work, "d.json", GOOD);
    let (code, out, _) = call(&["list", &f]);
    assert_eq!(code, 0);
    assert_eq!(
        out,
        "Acme X1 [verified] references: R1 R2\n# 1 components\n"
    );
}

#[test]
fn list_shows_a_dash_when_there_are_no_references() {
    let work = tempfile::tempdir().unwrap();
    let f = write_in(
        &work,
        "d.json",
        r#"{"components":[{"manufacturer":"A","part_number":"B"}]}"#,
    );
    let (_, out, _) = call(&["list", &f]);
    assert!(out.starts_with("A B [draft] references: -\n"));
}

#[test]
fn list_of_an_empty_database_prints_only_the_count() {
    let work = tempfile::tempdir().unwrap();
    let f = write_in(&work, "d.json", r#"{"components":[]}"#);
    assert_eq!(
        call(&["list", &f]),
        (0, "# 0 components\n".to_string(), String::new())
    );
}

#[test]
fn check_of_an_empty_database_is_clean() {
    let work = tempfile::tempdir().unwrap();
    let f = write_in(&work, "d.json", r#"{"components":[]}"#);
    let (code, out, _) = call(&["check", &f]);
    assert_eq!((code, out.as_str()), (0, "# 0 components, 0 findings\n"));
}

#[test]
fn a_finding_line_has_the_code_the_part_the_field_and_the_message() {
    let work = tempfile::tempdir().unwrap();
    let text = r#"{"components":[{"manufacturer":"A","part_number":"B","origin":{"kind":"manual"},"licence":"x",
        "fields":{"vcc":[{"value":"1","method":"measured","source":""}]}}]}"#;
    let f = write_in(&work, "d.json", text);
    let (code, out, _) = call(&["check", &f]);
    assert_eq!(code, 1);
    assert!(out.starts_with("PHOENIX-CINT-002 A B [vcc]: an observation has no source\n"));
    assert!(out.ends_with("# 1 components, 1 findings\n"));
}

#[test]
fn show_prints_the_component_as_json_and_exit_1_when_missing() {
    let work = tempfile::tempdir().unwrap();
    let f = write_in(&work, "d.json", GOOD);
    let (code, out, _) = call(&["show", &f, " ACME ", "x1"]);
    assert_eq!(code, 0);
    assert!(out.contains("\"part_number\": \"X1\""));
    let (code, out, err) = call(&["show", &f, "acme", "zzz"]);
    assert_eq!((code, out.as_str(), err.as_str()), (1, "", ""));
}

#[test]
fn export_of_a_good_database_is_the_json_with_a_line_feed() {
    let work = tempfile::tempdir().unwrap();
    let f = write_in(&work, "d.json", GOOD);
    let (code, out, _) = call(&["export", &f]);
    assert_eq!(code, 0);
    let back = Database::from_json(&out, &cfg()).unwrap();
    assert_eq!(back.components.len(), 1);
    assert!(out.ends_with("}\n"));
}

#[test]
fn a_missing_database_file_is_an_error_line_and_exit_2() {
    let (code, out, err) = call(&["check", "no-such-database-here.json"]);
    assert_eq!(code, 2);
    assert!(out.is_empty());
    assert!(err.starts_with("error: cannot read no-such-database-here.json"));
}

#[test]
fn an_empty_database_file_is_unreadable_not_a_panic() {
    let work = tempfile::tempdir().unwrap();
    let f = write_in(&work, "empty.json", "");
    let (code, _, err) = call(&["check", &f]);
    assert_eq!(code, 2);
    assert!(err.contains("the database cannot be read"));
}

#[test]
fn a_database_file_above_the_byte_limit_is_refused_at_the_edge() {
    let work = tempfile::tempdir().unwrap();
    let f = write_in(&work, "d.json", GOOD);
    let len = GOOD.len();
    let ok = write_in(&work, "ok.cfg", &format!("max_database_bytes = {len}\n"));
    let tight = write_in(
        &work,
        "tight.cfg",
        &format!("max_database_bytes = {}\n", len - 1),
    );
    assert_eq!(call(&["--config", &ok, "check", &f]).0, 0);
    let (code, _, err) = call(&["--config", &tight, "check", &f]);
    assert_eq!(code, 2);
    assert!(err.contains("limit exceeded"));
}

#[test]
fn a_bad_settings_file_exits_2_and_names_the_line() {
    let work = tempfile::tempdir().unwrap();
    let f = write_in(&work, "d.json", GOOD);
    let cfg_file = write_in(&work, "bad.cfg", "# c\nnonsense\n");
    let (code, _, err) = call(&["--config", &cfg_file, "check", &f]);
    assert_eq!(code, 2);
    assert!(err.contains("configuration line 2"));
}

#[test]
fn a_missing_settings_file_exits_2() {
    let work = tempfile::tempdir().unwrap();
    let f = write_in(&work, "d.json", GOOD);
    assert_eq!(call(&["--config", "no-such.cfg", "check", &f]).0, 2);
    assert_eq!(call(&["--config", "only-one"]).0, 2);
}

#[test]
fn check_csv_of_a_good_file_says_the_counts_and_writes_nothing() {
    let work = tempfile::tempdir().unwrap();
    let f = write_in(
        &work,
        "p.csv",
        &String::from_utf8(csv_of("R1,1,0402,A,P,\nR2,1,0402,A,P,\n")).unwrap(),
    );
    let (code, out, err) = call(&["check-csv", &f]);
    assert_eq!(code, 0);
    assert_eq!(out, "# 2 rows, 1 components, no problems\n");
    assert!(err.is_empty());
    assert_eq!(fs::read_dir(work.path()).unwrap().count(), 1);
}

#[test]
fn check_csv_of_a_bad_file_prints_each_issue_with_its_code_and_exits_2() {
    let work = tempfile::tempdir().unwrap();
    let f = write_in(
        &work,
        "p.csv",
        &String::from_utf8(csv_of("R1,1,0402,,,\n")).unwrap(),
    );
    let (code, out, err) = call(&["check-csv", &f]);
    assert_eq!(code, 2);
    assert!(out.is_empty());
    assert!(err.contains("EH3D-CINT-110 line 2: the column `manufacturer` is empty"));
    assert!(err.ends_with("# 2 problems, nothing was imported\n"));
}

#[test]
fn check_csv_above_the_byte_limit_gives_the_too_large_code() {
    let work = tempfile::tempdir().unwrap();
    let f = write_in(
        &work,
        "p.csv",
        &String::from_utf8(csv_of("R1,1,0402,A,P,\n")).unwrap(),
    );
    let c = write_in(&work, "s.cfg", "csv_max_bytes = 10\n");
    let (code, _, err) = call(&["--config", &c, "check-csv", &f]);
    assert_eq!(code, 2);
    assert!(err.contains("EH3D-CINT-101: the file is larger than 10 bytes"));
}

#[test]
fn import_writes_a_canonical_file_that_the_check_reads_back() {
    let work = tempfile::tempdir().unwrap();
    let f = write_in(
        &work,
        "p.csv",
        &String::from_utf8(csv_of("R2,1,0402,B,Q,\nR1,1,0402,A,P,\n")).unwrap(),
    );
    let target = work.path().join("db.json");
    let t = target.to_str().unwrap();
    let (code, out, _) = call(&["import", &f, t]);
    assert_eq!(code, 0);
    assert!(out.starts_with("# 2 rows, 2 components written to "));
    let text = fs::read_to_string(&target).unwrap();
    let d = Database::from_json(&text, &cfg()).unwrap();
    assert_eq!(d.to_canonical_json().unwrap(), text);
    let (code, out, _) = call(&["check", t]);
    assert_eq!(code, 1);
    assert!(out.contains("PHOENIX-CINT-006"));
}

#[test]
fn import_never_replaces_a_file_without_the_option() {
    let work = tempfile::tempdir().unwrap();
    let f = write_in(
        &work,
        "p.csv",
        &String::from_utf8(csv_of("R1,1,0402,A,P,\n")).unwrap(),
    );
    let t = write_in(&work, "db.json", "keep");
    let (code, _, err) = call(&["import", &f, &t]);
    assert_eq!(code, 2);
    assert!(err.contains("EH3D-CINT-112"));
    assert_eq!(fs::read_to_string(&t).unwrap(), "keep");
    assert_eq!(call(&["import", &f, &t, "--overwrite"]).0, 0);
    assert_ne!(fs::read_to_string(&t).unwrap(), "keep");
}

#[test]
fn a_failed_import_leaves_no_file_behind() {
    let work = tempfile::tempdir().unwrap();
    let f = write_in(&work, "p.csv", "garbage");
    let target = work.path().join("db.json");
    let (code, _, _) = call(&["import", &f, target.to_str().unwrap()]);
    assert_eq!(code, 2);
    assert!(!target.exists());
}

#[test]
fn import_twice_with_overwrite_gives_the_same_bytes() {
    let work = tempfile::tempdir().unwrap();
    let f = write_in(
        &work,
        "p.csv",
        &String::from_utf8(csv_of("R2,1,0402,B,Q,\nR1,1,0402,A,P,\n")).unwrap(),
    );
    let t = work.path().join("db.json");
    let t = t.to_str().unwrap();
    assert_eq!(call(&["import", &f, t]).0, 0);
    let one = fs::read(t).unwrap();
    assert_eq!(call(&["import", &f, t, "--overwrite"]).0, 0);
    assert_eq!(one, fs::read(t).unwrap());
}

#[test]
fn the_licence_setting_makes_an_imported_database_exportable() {
    let work = tempfile::tempdir().unwrap();
    let f = write_in(
        &work,
        "p.csv",
        &String::from_utf8(csv_of("R1,1,0402,A,P,\n")).unwrap(),
    );
    let c = write_in(&work, "s.cfg", "import_licence = CC0\n");
    let t = work.path().join("db.json");
    let t = t.to_str().unwrap();
    assert_eq!(call(&["--config", &c, "import", &f, t]).0, 0);
    assert_eq!(call(&["check", t]).0, 0);
    assert_eq!(call(&["export", t]).0, 0);
}
