# EH3D-COMPONENT-INTELLIGENCE: decisions that apply here

Decisions of the suite that shape this project, written as project documentation. A decision marked proposed is not final: this project must not rely on it silently, and where a number or a limit depends on it, the code will keep it as one named constant.

## Independent repositories related by contracts

State: **accepted**.

Every repository of the suite is independent. Repositories relate through versioned contracts and build next to each other as sibling folders. Programs never depend on each other: they exchange work through project files or through the server. The design tool does not depend at build time on the satellites that depend on it; it reaches them through their command line or through server tasks. A lower layer never depends on a higher one.

## Stable diagnostic codes

State: **proposed**.

Failures that a reader or a check cannot hide become diagnostics with a stable code of the form `PHOENIX-<AREA>-<NNN>` (the registry is kept in the board tool). A code is never reused or renumbered, its message may be translated but the code and field names are not, and a repository never keeps a private list: it asks for numbers in the registry in the same change that uses them, together with a test.

## Canonical JSON and numbers in new formats

State: **proposed**.

New JSON files use the JSON Canonicalization Scheme (RFC 8785) plus UTF-8 without BOM and a final line feed. Design values are integers with the unit in the name; exchange values in millimetres are quantised before writing; physical results with uncertainty are compared numerically, not byte for byte. The existing `.eh3dproj` 1.0.0 behaviour does not change.

## Private designs stay private

State: **accepted**.

The real projects of the author are never committed, pasted into issues or sent to a service. Tests use synthetic cases or minimal anonymised reproducers that the owner approved. Scripts that run over a private corpus use redacted mode, which reports only codes, counts, levels and hashes.

## Files in the project, no database server

State: **accepted**.

The database is a JSON file kept next to the project, in a stable form (`DATABASE_FORMAT.md`): it can be read, compared and versioned like any other file. There is no database server. A file is written atomically and an existing file is never replaced without an explicit order.

## A plain CSV list of parts as the first import format

State: **accepted**.

The first import format is a CSV file with fixed columns (`CSV_FORMAT.md`): a mandatory header, a configurable separator, quotes, UTF-8 with or without a byte order mark, LF or CRLF. Its refusals carry codes `EH3D-CINT-NNN`; the checks of a database keep `PHOENIX-CINT-NNN`.
