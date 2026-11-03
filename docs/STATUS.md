# EH3D-COMPONENT-INTELLIGENCE: status

## Current state

EH3D-COMPONENT-INTELLIGENCE is planned. Version 0.0.1 holds only the base that every repository of the suite carries (manifest, README files in eight languages, changelog, CI, shared checks) and the written scope, requirements, dependencies, threat model and decisions. It also has a Rust library and a command line that read a JSON database of components and check it (duplicate identities, observations without a source, sources that disagree, records marked verified while something is pending, missing origin or licence) and refuse to export data without a licence. They have run only on data made by their own tests; nothing has been run on a machine with real data, and no requirement is validated.

## Capabilities

The official state of each capability is in REQUIREMENTS.md: 6 requirements, all `prototype`. This page never states a state that REQUIREMENTS.md does not.

## Open questions

* Storage form of the database (files in the project, a local database, or both).

## Needs from other repositories

* EH3D-COMMON: container and diagnostics

## Known technical debt and omitted tests

14 tests of the Rust code and the 3 of the base; the Rust tests build their own databases and never touch real data. Not tested: very large databases, and the files of a database edited by hand with unusual characters. `cargo deny` is not run yet.
