# EH3D-COMPONENT-INTELLIGENCE: threat model

Basic model for early code. The rows marked partial have a mitigation written and tests for it, run only on data made by the tests themselves; the rows marked planned have nothing yet. No row is assessed against real data.

| Threat | Planned mitigation | Planned test | State | Residual risk |
|---|---|---|---|---|
| Wrong or poisoned component data imported from a third party | provenance and confidence recorded; the import records its file as the origin, keeps disagreeing values side by side instead of choosing, and never replaces an existing file without an explicit order | negative tests of the import and of the writing | partial | the content of an imported file is not judged: wrong values are still wrong values |
| Licence violation by redistributing manufacturer data | licence field required for export; the import never invents a licence | export tests | partial | the text of the licence is not interpreted |
| Corrupt or malicious project or input files (oversized entries, path tricks, deep nesting, resource exhaustion) | size, row, column and field limits in the settings file; the CSV reader is strict and bounded | hostile samples written by hand; no fuzzing yet | partial | the JSON database is bounded by size and counts, not by nesting depth; no fuzzing |
| Temporary files, logs and reports that reveal private designs | no private data in outputs; redaction | automated pattern search | planned | not assessed: nothing is implemented |
| Compromised dependency or binary | pinned versions, licence and advisory checks, software bill of materials for releases | CI checks | planned | not assessed: nothing is implemented |
| Power loss or forced close during a save | atomic write: temporary file, then rename; old file never deleted first | tests of the writer; no interruption tests | partial | a power loss is not simulated; on a file system without hard links the check that the file does not exist and the move are two steps |

Minimum negative tests once there is code: hostile files and resource limits; secrets absent from logs, errors and reports; pinned dependencies and licence review before publishing. Report a vulnerability privately as described in SECURITY.md, without attaching private designs.
