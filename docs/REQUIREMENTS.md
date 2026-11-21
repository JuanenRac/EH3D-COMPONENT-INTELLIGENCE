# EH3D-COMPONENT-INTELLIGENCE: requirements

This register is the single source of the state of each capability: `STATUS.md` and the README files cite it and never state a state that it does not. A state is one of `planned`, `prototype`, `implemented` and `validated`. `validated` means that a person has run the capability on a real machine and the evidence of the manifest says `bench` or `field`; nothing here is validated, because nothing of this program has been run with the parts list of a real project or a real data sheet.

The register has 35 requirements: 27 implemented, 6 prototype, 2 planned and 0 validated. By progress: 27 done, 6 partial, 1 pending and 1 blocked. High coverage is not proof of correctness, so each row says how it is checked: the test that names it, and what it shows.

## How to read the register

* **ID:** `CINT-NNN`, never reused and never renumbered; a new capability takes the next number.
* **State:** `implemented` is code that is complete and tested on a computer, `prototype` is part of the capability, `planned` is not written.
* **Progress:** `done`, `partial`, `pending` (can be written without hardware) or `blocked` (waits for hardware or for another program of the suite).
* **Evidence:** `simulator` when the test runs against a seeded twin of the program that exports a list of parts, with damage injected into its file, `host-test` when it checks a calculation, a reader or a screen-less piece, `none` when nothing is written yet.
* **Test:** the acceptance criterion: `file::name` is the test `name` of `tests/file.rs` (`src/file::name` is a test inside `src/file.rs`), which must exist. For a capability that is not written the cell says `planned:` and the name of the test that will show it; the day that test exists, the row must change.
* **Limits:** what is not done or not proven; a row that is `partial` or `blocked` always says it.

The check is `tests/reqs.rs`: it reads this table, finds each test and fails when a state, a test or a count does not agree.

## The database and its checks

| ID | Requirement | State | Progress | Evidence | Test | Limits |
|---|---|---|---|---|---|---|
| CINT-001 | Component record with an exact identity: manufacturer and part number. | prototype | partial | host-test | `src/database::the_same_identity_twice_is_found_whatever_the_letter_case` | It compares manufacturer and part number in lower case without surrounding spaces; it reports a duplicate and never merges two records. |
| CINT-002 | Provenance for every field. | prototype | partial | host-test | `src/database::a_field_without_a_source_is_flagged` | It flags an observation whose source is empty; it cannot tell whether the source named is real. |
| CINT-003 | Show the conflict when two sources disagree. | prototype | partial | host-test | `src/database::two_sources_that_disagree_show_both_values` | It compares the values as text after trimming spaces, so `3.3 V` and `3,3 V` count as different; units are not understood. |
| CINT-004 | Confidence state and fields pending review: a record with pending fields is never verified. | prototype | partial | host-test | `src/database::a_record_with_pending_fields_is_never_verified` | A record marked verified is flagged when a field is pending, in conflict, without a source or only inferred (the last rule is a setting); the status is set by a person, nothing here sets it. |
| CINT-005 | Manual and imported fill: every record says where it came from. | prototype | partial | host-test | `src/database::an_import_must_say_what_it_came_from_and_a_record_needs_an_origin` | The CSV list of parts is the only import format and it makes a new file; a database cannot be merged into an existing one. |
| CINT-006 | Record the licence of imported data and refuse to export data without one. | prototype | partial | host-test | `src/database::data_without_a_licence_is_not_exported` | A setting can switch the rule off; the text of the licence is not interpreted. |
| CINT-007 | A clean record has no findings. | implemented | done | host-test | `src/database::a_clean_record_has_no_findings` | - |
| CINT-008 | The findings come in a fixed order. | implemented | done | host-test | `src/database::findings_come_in_a_fixed_order` | - |
| CINT-009 | Bad records and bad JSON are refused without a panic. | implemented | done | host-test | `src/database::bad_records_and_bad_json_are_refused_without_panicking` | - |
| CINT-010 | The codes of the problems are stable and distinct, and the text names the line only when there is one. | implemented | done | host-test | `src/issue::codes_are_stable_and_distinct` | - |

## The CSV reader and the import

| ID | Requirement | State | Progress | Evidence | Test | Limits |
|---|---|---|---|---|---|---|
| CINT-011 | LF and CRLF give the same table and the header is normalised. | implemented | done | host-test | `src/csv::lf_and_crlf_give_the_same_table_and_the_header_is_normalised` | - |
| CINT-012 | Quotes hold separators, quotes and line breaks. | implemented | done | host-test | `src/csv::quotes_hold_separators_quotes_and_line_breaks` | - |
| CINT-013 | A file that is not UTF-8 names the line. | implemented | done | host-test | `src/csv::a_file_that_is_not_utf8_names_the_line` | - |
| CINT-014 | Every limit of the CSV reader is enforced. | implemented | done | host-test | `src/csv::every_limit_is_enforced` | - |
| CINT-015 | Lines with the same part become one component. | implemented | done | host-test | `src/import::lines_with_the_same_part_become_one_component` | - |
| CINT-016 | Imported records have no licence unless the settings give one. | implemented | done | host-test | `src/import::imported_records_have_no_licence_unless_the_settings_give_one` | - |
| CINT-017 | Two values for one part are kept and the check reports the conflict. | implemented | done | host-test | `src/import::two_values_for_one_part_are_kept_and_the_check_reports_the_conflict` | - |
| CINT-018 | Empty required values and repeated references are listed with their line. | implemented | done | host-test | `src/import::empty_required_values_and_repeated_references_are_listed_with_their_line` | - |
| CINT-019 | The header is checked before any row. | implemented | done | host-test | `src/import::the_header_is_checked_before_any_row` | - |
| CINT-020 | The canonical JSON does not depend on the order of the lines. | implemented | done | host-test | `src/import::the_canonical_json_does_not_depend_on_the_order_of_the_lines` | - |
| CINT-021 | An existing database file is kept unless the caller orders the replacement. | implemented | done | host-test | `src/store::an_existing_file_is_kept_unless_the_caller_orders_the_replacement` | - |

## The command line and the settings

| ID | Requirement | State | Progress | Evidence | Test | Limits |
|---|---|---|---|---|---|---|
| CINT-022 | `check`, `show` and `export` exit with the code that tells the result. | implemented | done | host-test | `src/cli::check_show_and_export_exit_codes` | - |
| CINT-023 | Bad input and bad arguments exit with 2. | implemented | done | host-test | `src/cli::bad_input_and_bad_arguments_exit_2` | - |
| CINT-024 | `import` writes the database and `check` and `list` read it back. | implemented | done | host-test | `src/cli::import_writes_the_database_and_check_and_list_read_it_back` | - |
| CINT-025 | A bad CSV lists its problems with code and line and writes nothing. | implemented | done | host-test | `src/cli::a_bad_csv_lists_its_problems_with_code_and_line_and_writes_nothing` | - |
| CINT-026 | An existing database is replaced only with the explicit order. | implemented | done | host-test | `src/cli::an_existing_database_is_replaced_only_with_the_explicit_order` | - |
| CINT-027 | The examples are read whatever their encoding and line endings. | implemented | done | host-test | `src/cli::the_examples_are_read_whatever_their_encoding_and_line_endings` | The examples are small files made by hand; no list from a real project has been read. |
| CINT-028 | The default settings file says the defaults and a bad line is refused. | implemented | done | host-test | `src/config::the_default_file_says_the_defaults_and_the_example_is_that_file` | - |

## Simulation of the export of a list of parts

| ID | Requirement | State | Progress | Evidence | Test | Limits |
|---|---|---|---|---|---|---|
| CINT-029 | A seeded twin gives the same file for the same seed and the file it makes is imported whole. | implemented | done | simulator | `src/simulation::a_file_made_by_the_twin_is_imported_whole` | The twin imitates the form of a list, not a real exporter. |
| CINT-030 | Every kind of damage is refused with its code. | implemented | done | simulator | `src/simulation::every_damage_is_refused_with_its_code` | - |
| CINT-031 | The size limit holds to the byte. | implemented | done | simulator | `src/simulation::the_size_limit_holds_to_the_byte` | - |
| CINT-032 | A damaged byte never panics and always gives the same answer. | implemented | done | simulator | `src/simulation::a_damaged_byte_never_panics_and_always_gives_the_same_answer` | - |
| CINT-033 | Asking for the list again recovers from a damaged file. | implemented | done | simulator | `src/simulation::asking_for_the_list_again_recovers_from_a_damaged_file` | - |
| CINT-034 | Link a record to the symbols and the footprints of the design tool. | planned | blocked | none | `planned:footprints::a_record_points_to_a_symbol_and_a_footprint_of_the_design_tool` | Waits for the design tool to publish the model of a symbol and a footprint. |
| CINT-035 | Merge an import into a database that already exists without overwriting verified data. | planned | pending | none | `planned:merge::an_import_merged_into_a_database_never_overwrites_a_verified_record` | Not written: the rule for what wins has not been decided. |
