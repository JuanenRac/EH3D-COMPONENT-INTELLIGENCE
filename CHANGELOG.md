# Changelog

All notable changes to this project are documented here.

## [Unreleased]

- **The Rust jobs keep a cache of the build:** `rust.yml` uses `Swatinem/rust-cache`, pinned by its commit, after the installation of Rust, so a run that changes little does not build the dependencies again.
- **The CI is split in two workflows:** the validation of the repository (manifest, documentation, pictures, tools) stays in `ci.yml` and the Rust jobs move to `rust.yml`, which does not run for a change that only touches Markdown files or pictures, so a change of the documentation no longer builds the code on two systems. The names of the jobs are the same.
- **The Rust toolchain is pinned:** `rust-toolchain.toml` fixes the version of Rust (the same one in every repository of the suite) with `rustfmt` and `clippy`, the workflows install that version and not `stable`, so a new release of Rust cannot turn the CI red by itself.
## [0.0.3] - Import of components from a CSV file stored in the project and the safe write of the database

- **Import of a CSV list of parts:** the new `import` command makes a database file from a CSV with the fixed columns `reference`, `value`, `package`, `manufacturer`, `mpn` and `datasheet`; a mandatory header, a configurable separator, quotes, UTF-8 with or without a byte order mark and LF or CRLF lines are accepted. The format is written in `docs/CSV_FORMAT.md`.
- **One component per part:** lines with the same manufacturer and part number become one component that lists all its references; the origin is the file, the observations are transcribed from it, the licence is never invented, and two different values for the same field stay side by side so that `check` reports the conflict.
- **Bounded, strict reading with stable refusals:** the CSV reader limits bytes, rows, columns, the length of a field and the number of problems listed, all in the settings file, and refuses a bad file as a whole with the codes `EH3D-CINT-101` to `EH3D-CINT-113` and the line of each problem; nothing is written when anything is wrong.
- **New commands `check-csv` and `list`:** `check-csv` reads a CSV and reports every problem without writing anything; `list` prints one line per component with its status and references.
- **Safe writing of the database file:** the file is a plain JSON file next to the project in a stable form (sorted, two-space indentation, final line feed), written whole to a temporary file and then put in place; an existing file is never replaced without `--overwrite` (`EH3D-CINT-112`). The format is written in `docs/DATABASE_FORMAT.md`.
- **Settings file with an example that cannot drift:** `Config::default_file()` gives the text of the settings with every default, and `examples/settings.cfg` is checked to be exactly that text; `.cfg` and `.csv` files keep their line endings (`-text`).
- **Examples and tests:** three small CSV files (plain, with quotes, empty fields, a byte order mark and CRLF lines, and with a semicolon separator) and 32 new tests with hand-computed values: quotes, empty fields, duplicate references, byte order mark, line endings, every limit, the header, the canonical form, refusal to overwrite and the exit codes of the commands.
- **Documentation brought up to date:** the README files in eight languages, `STATUS`, `NOT_YET`, `REQUIREMENTS`, `DECISIONS` (storage in files, CSV as the first import), `THREAT_MODEL`, and the architecture and structure pictures describe what the code now does.

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
