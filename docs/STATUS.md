# EH3D-COMPONENT-INTELLIGENCE: status

## Current state

EH3D-COMPONENT-INTELLIGENCE is early code on a computer: a Rust library and a command line (`check`, `show`, `export`, `list`, `check-csv`, `import`) that read a JSON database of components and check it, refuse to export data without a licence, and import a CSV list of parts into a new database file. The import is strict and bounded and reports every problem with a code and a line. A seeded twin of the program that exports a list of parts damages the file in seven ways. All of it has run only on data made by its own tests and on the small files of `examples/`; nothing has been run with real data and no requirement is validated.

## Capabilities

The official state of each capability is in `REQUIREMENTS.md`. The register has 35 requirements: 27 implemented, 6 prototype, 2 planned and 0 validated. By progress: 27 done, 6 partial, 1 pending and 1 blocked. This page never states a state that `REQUIREMENTS.md` does not.

## Open questions

* The link with the symbols and footprints of the design tool (not decided).
* Merging an import into a database that already exists (not written; see `NOT_YET.md`).

## What it needs from other repositories

* EH3D-COMMON: the container and the diagnostics.
* EH3D-PHOENIX-PCB: the symbols and footprints a record will point to.

## Known technical debt and omitted tests

* 212 tests of the Rust code (nine of them keep this register true) and the 3 of the base; the Rust tests build their own databases and read only the small files of `examples/`, never real data.
* Eleven of the tests run the simulation (`src/simulation.rs`): the seven kinds of damage, the limit of bytes tested to the byte, a list of three thousand lines, a seeded byte-damage run that must never panic, the recovery and the safe writing of the database file.
* Not tested: a power loss during a write, and the files of a database edited by hand with unusual characters; the CSV reader has no coverage-guided fuzzing.
* `cargo deny` is not run yet.
