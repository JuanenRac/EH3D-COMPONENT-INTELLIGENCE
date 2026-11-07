# EH3D-COMPONENT-INTELLIGENCE: requirements

This register is the single source of the state of each capability. Status values: planned, prototype, implemented, validated. Every requirement is `prototype` today (code and tests exist, only on data made by the tests); none has been used on real data or validated. High test coverage is not proof of geometric or electrical correctness, so each requirement says how it will be checked and what result is expected.

| Requirement | Description | Status | How it will be checked | Expected result | Known limits |
|---|---|---|---|---|---|
| CINT-001 | Component record with exact identity. | prototype | automatic | Two records with the same manufacturer and part number are rejected or merged explicitly. | Compares manufacturer and part number in lower case without surrounding spaces; it reports a duplicate and never merges two records. Only tried on data made by the tests. |
| CINT-002 | Provenance for every field. | prototype | automatic | A field without provenance is flagged. | Flags an observation whose source is empty; it cannot tell whether the source named is real. |
| CINT-003 | Show conflicts between two sources. | prototype | automatic | Both values and sources are listed. | Compares the values as text after trimming spaces, so `3.3 V` and `3,3 V` count as different; units are not understood. |
| CINT-004 | Confidence state and fields pending review. | prototype | automatic | A record with pending fields is never marked verified. | A record marked verified is flagged when a field is pending, in conflict, without a source or only inferred (this last rule has a setting). The status is set by a person; nothing here sets it. |
| CINT-005 | Manual and imported fill. | prototype | automatic | Imports record their origin; a file that does not follow the format is refused with a code and a line and nothing is written. | A record must say whether it was made by hand or imported, and an import must name its source. The import of a CSV list of parts (`CSV_FORMAT.md`) records the file as the origin of every record it makes; it is the only import format, it makes a new file and cannot be merged into an existing database. Tried only on the small files of `examples/` and on data made by the tests. |
| CINT-006 | Record the licence of imported data. | prototype | automatic | Data without a recorded licence is not exportable. | Export is refused while any record has no licence (a setting can switch the rule off). The text of the licence is not interpreted. |

Rules: a claim of correctness names the corpus, the tolerance and the tool version it holds for. A phase is complete when the measured rate does not drop, every failure is classified and fixed or documented, and the measured figure is reported.
