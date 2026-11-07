# EH3D-COMPONENT-INTELLIGENCE: status

## Current state

EH3D-COMPONENT-INTELLIGENCE is planned. Version 0.0.1 holds only the base that every repository of the suite carries (manifest, README files in eight languages, changelog, CI, shared checks) and the written scope, requirements, dependencies, threat model and decisions. It also has a Rust library and a command line that read a JSON database of components and check it (duplicate identities, observations without a source, sources that disagree, records marked verified while something is pending, missing origin or licence), refuse to export data without a licence, and import a CSV list of parts into a new database file (`import`, `check-csv`, `list`; the format is in `CSV_FORMAT.md`, the file in `DATABASE_FORMAT.md`). The import is strict and bounded, reports every problem with a code `EH3D-CINT-NNN` and a line, and never replaces an existing file without `--overwrite`. All of it is tested only on data made by its own tests and on the small files of `examples/`; nothing has been run on a machine with real data, and no requirement is validated.

## Capabilities

The official state of each capability is in REQUIREMENTS.md: 6 requirements, all `prototype`. This page never states a state that REQUIREMENTS.md does not.

## Open questions

* The link with the symbols and footprints of the design tool (not decided).
* Merging an import into a database that already exists (not written; see `NOT_YET.md`).

## Needs from other repositories

* EH3D-COMMON: container and diagnostics

## Known technical debt and omitted tests

46 tests of the Rust code and the 3 of the base; the Rust tests build their own databases and read only the small files of `examples/`, never real data. The CI runs them. Not tested: files near the size limits, very large databases, a power loss during a write, and the files of a database edited by hand with unusual characters; the CSV reader has no fuzzing. `cargo deny` is not run yet.
