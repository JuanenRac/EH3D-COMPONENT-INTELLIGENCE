# Changelog

All notable changes to this project are documented here.

## [Unreleased]

## [0.0.2] - First release: the library, the command line, the stable error codes and the tests

- **Brand:** Add the application icon and the logos of the program.
- **Shared checks updated:** the validator also compares the badge and the Version row of the README with the manifest, fails when one of its checks cannot run, and the CHANGELOG step of the workflow reads the base branch from the environment.

- Add the library: components identified by manufacturer and part number, the observations of every field with how they were obtained and their source, page, table, revision and date, the confidence state, the pending fields, the origin and the licence of the data.
- Find and report, with stable codes `PHOENIX-CINT-001` to `PHOENIX-CINT-007`: duplicate identities, observations without a source, sources that disagree, records marked verified while something is pending, records without an origin, records without a licence and malformed records.
- Add the command line (`check`, `show`, `export`) and a settings file; exporting is refused while a record has no licence.
- Run format, lints and tests of the Rust code on Linux and Windows in the CI.
- **The workflows are hardened:** every action is pinned by its commit with its version in a comment, and the checkouts keep no credentials.

## [0.0.1] - The base of the repository

- **The base that every repository of the suite has:** the manifest, the README files in eight languages, this changelog, the CI workflow, the issue and pull-request templates, and the shared checks of EH3D-DOCS (the repository check and the drawers of the two README pictures).
- **A planned project, described honestly:** the scope, the requirements (all planned), the relations with other repositories, the threat model, the decisions that apply and the rules for writing code are in `docs/`; there is no code yet.
