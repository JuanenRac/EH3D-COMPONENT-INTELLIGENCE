# EH3D-COMPONENT-INTELLIGENCE: the database file

The database is **one JSON file kept next to the project** (for example `parts.json` beside the design). There is no database server and no hidden folder: the file can be read, compared and kept under version control like any other file of the project.

## Shape

```json
{
  "components": [
    {
      "manufacturer": "Yageo",
      "part_number": "RC0603FR-0710KL",
      "status": "draft",
      "pending": [],
      "origin": { "kind": "import", "source": "parts-small.csv" },
      "licence": null,
      "references": ["R1", "R2"],
      "fields": {
        "value": [
          { "value": "10 kΩ", "method": "transcribed", "source": "parts-small.csv", "page": null, "table": null, "revision": null, "consulted": null }
        ]
      }
    }
  ]
}
```

| Key | Meaning |
|---|---|
| `manufacturer`, `part_number` | The identity. Two records with the same pair (letter case and surrounding spaces ignored) are a finding (`PHOENIX-CINT-001`). |
| `status` | `draft` or `verified`. A person sets it; nothing here does. |
| `pending` | The fields still to review. A record with pending fields must not be `verified`. |
| `origin` | `{"kind": "manual" or "import", "source": ...}`; an import must name its source. |
| `licence` | The licence of the data, as written by the person who reviewed it; empty means it cannot be exported. |
| `references` | Reference designators that use the part; left out of the file when empty. Added by the import. |
| `fields` | By field name, a list of observations: `value`, `method` (`extracted`, `transcribed`, `inferred`, `verified`, `measured`), `source`, and optionally `page`, `table`, `revision`, `consulted` (text, as the person wrote it). |

## Canonical form

The commands that write a file (`import`) write it in a fixed form, so that the same content always gives the same bytes:

* components sorted by identity (manufacturer, then part number, in lower case), then by the spelling;
* `references` sorted; the observations of a field sorted by value and then source; the fields sorted by name;
* two-space indentation, UTF-8 without a byte order mark, and one final line feed.

This is a stable form of this project, not the canonicalisation scheme of RFC 8785.

## Writing is safe

* The file is written whole to a temporary file in the same folder, flushed to the disk and then put in place; a failure never leaves a half-written database, and the temporary file is removed.
* An existing file is **never** replaced without `--overwrite`; without it the command stops with `EH3D-CINT-112` and the file is untouched.
* A folder that does not exist is not created (`EH3D-CINT-113`).

## Reading

A database larger than `max_database_bytes`, with more than `max_components` records, or with more than `max_observations` observations in one field is refused. Unknown keys are ignored; a file that is not valid JSON of this shape is refused with a message.
The files are read as written by a person too: `check` does not repair anything.
