# EH3D-COMPONENT-INTELLIGENCE: scope

Status: **early code**. EH3D-COMPONENT-INTELLIGENCE is a library and a command line that check a JSON database of components and import a CSV list of parts into one. It has been tried on data made by its own tests and on the small files of `examples/`. This page says what it is for, what it will not do and where it sits among the other programs of the suite.

## Purpose

The technical database of the electronic components of a design, with the metadata and the provenance of each value: who made the part, where each value came from, whether two sources disagree and which fields still wait for a person to review them.

## What it does

* Reads a database (a JSON file, `DATABASE_FORMAT.md`) within a size limit and checks it: duplicate identities (manufacturer and part number, whatever the letter case), observations without a source, sources that disagree, records marked verified while something is pending, and a missing origin or licence.
* Shows a record with the values and sources of every conflict, and lists the records.
* Refuses to export data without a licence (a setting can switch the rule off) and never merges two records.
* Imports a CSV list of parts (`CSV_FORMAT.md`) into a new database file: strict and bounded, every problem reported with a code `EH3D-CINT-NNN` and a line, nothing written when the file is bad, and an existing file replaced only with the explicit order.
* Checks a CSV file without importing it (`check-csv`), and reads the files whatever their encoding marker and line endings.
* Runs a seeded twin of the program that exports a list of parts, with seven kinds of damage injected into its file and a recovery that asks for the list again.

## What it will not do

* No tool replaces a component without checking its pin-out, its package, its critical parameters and its operating conditions.
* It redistributes no manufacturer data before the licence has been reviewed, and bundles no data sheets.
* It makes no network access unless a task explicitly allows it, and it has no artificial intelligence in version 1.
* It does not merge an import into a database that already exists, and it reads no format of another design program (see `NOT_YET.md`).
* It does not claim that anything was proven with real data: the evidence of every capability is `simulator` or `host-test`.

## Where it sits

* **It reads:** a database file (JSON), a CSV list of parts and a settings file (the size limits, the CSV separator, the licence of imported data and the export rule).
* **It writes:** the database that an import makes, through a temporary file and only into a new path unless `--overwrite` is given; it never writes to a database it only reads.
* **It is read by:** nothing reads it yet: the design tool will read the database for symbols and footprints once the link is decided.
* **It depends on:** nothing of the suite at build time; the container and the diagnostics come from EH3D-COMMON.

## What it never decides

Whether a part is right for a circuit, and the limits, the interlocks and the emergency stop of any machine, belong to the engineer who chooses the part. A record marked verified says that a person checked it; this program only refuses to mark it so while something is pending. Nothing here starts motion.

The requirements of the program, with the state of each and the test that shows it, are in `REQUIREMENTS.md`; the state in a paragraph is in `STATUS.md`.
