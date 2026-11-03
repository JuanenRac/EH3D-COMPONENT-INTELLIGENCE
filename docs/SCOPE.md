# EH3D-COMPONENT-INTELLIGENCE: scope

Status: **early code**. EH3D-COMPONENT-INTELLIGENCE has a library and a command line that check a JSON database; most of what this page lists is still planned. This page says what it is meant to do, what it will not do, and where it sits. Nothing has been run on real data.

## Purpose

EH3D-COMPONENT-INTELLIGENCE is planned as a technical database of electronic components with their metadata.

Technical database of electronic components with metadata and provenance for each value.

## What it is planned to do

* For each component: manufacturer and exact part number, data sheet and where it came from, symbol, footprint and pin mapping, simulation models, electrical and thermal limits, lifecycle and the date it was consulted, availability and alternatives, a confidence level and the fields still to review.
* Say how each datum was obtained: extracted, transcribed, inferred, verified against the data sheet, or measured, with page, table, revision and date.
* Show the conflict when two sources disagree.
* First version filled by hand or imported, with no artificial intelligence.

## What it will not do

* No tool replaces a component without checking pin-out, package, critical parameters and operating conditions.
* No manufacturer data is redistributed before its licence has been reviewed; no data sheets are bundled.
* No network access unless a task explicitly allows it.

## Planned shape

A library without a window, written in Rust (planned), with a command-line interface and, where it applies, a kind of task for the server. Programs of the suite exchange work through project files or through the server, never by depending on each other.

The package is a single crate (a library and a command line, no window) with `check`, `show` and `export`; the database is a JSON file. Importers and the link with the symbols and footprints of the design tool are not written.

## Where it sits

It belongs to the PHOENIX family of the EH3D Fabrication Suite: independent repositories that relate through versioned contracts. The design tool (EH3D-PHOENIX-PCB) does not depend at build time on the repositories that depend on it.
See DEPENDENCIES.md for the relations, REQUIREMENTS.md for the requirements and STATUS.md for the honest state.
