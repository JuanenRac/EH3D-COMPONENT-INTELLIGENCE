# EH3D-COMPONENT-INTELLIGENCE: requirements

This register is the single source of the state of each capability. Status values: planned, prototype, implemented, validated. Every requirement is `planned` today: nothing is implemented or verified. High test coverage is not proof of geometric or electrical correctness, so each requirement says how it will be checked and what result is expected.

| Requirement | Description | Status | How it will be checked | Expected result | Known limits |
|---|---|---|---|---|---|
| CINT-001 | Component record with exact identity. | planned | automatic | Two records with the same manufacturer and part number are rejected or merged explicitly. | Not started: no limits known yet. |
| CINT-002 | Provenance for every field. | planned | automatic | A field without provenance is flagged. | Not started: no limits known yet. |
| CINT-003 | Show conflicts between two sources. | planned | automatic | Both values and sources are listed. | Not started: no limits known yet. |
| CINT-004 | Confidence state and fields pending review. | planned | automatic | A record with pending fields is never marked verified. | Not started: no limits known yet. |
| CINT-005 | Manual and imported fill. | planned | automatic | Imports record their origin. | Not started: no limits known yet. |
| CINT-006 | Record the licence of imported data. | planned | automatic | Data without a recorded licence is not exportable. | Not started: no limits known yet. |

Rules: a claim of correctness names the corpus, the tolerance and the tool version it holds for. A phase is complete when the measured rate does not drop, every failure is classified and fixed or documented, and the measured figure is reported.
