# EH3D-COMPONENT-INTELLIGENCE: the CSV list of parts

The only import format of this version is a plain CSV file with one line per reference of a design (a bill of materials). Nothing is guessed:
a file that does not follow these rules is refused as a whole, with the code and the line of every problem, and nothing is written.

Try it with the files in `examples/`:

```bash
eh3d-component-intelligence check-csv examples/parts-small.csv
eh3d-component-intelligence import examples/parts-small.csv parts.json
eh3d-component-intelligence --config examples/semicolon.cfg import examples/parts-semicolon.csv parts.json
```

## Layout of the file

| Rule | What it means |
|---|---|
| Encoding | UTF-8, with or without a byte order mark. Anything else is refused (`EH3D-CINT-105`, with the line of the first bad byte). |
| Line ends | LF or CRLF, mixed freely. A carriage return that is not followed by a line feed is refused. |
| Separator | One character, `,` by default. Set it with `csv_separator` in the settings file (`;`, `|`, or the word `tab`). |
| Quotes | A field that starts with `"` ends at the next lone `"`; `""` inside stands for one `"`. Such a field may hold separators and line breaks. A `"` anywhere else (inside a field that does not start with one, or text after the closing `"`) is refused (`EH3D-CINT-106`). |
| Blank lines | Skipped. |
| Header | Mandatory: the first line. The names are matched without surrounding spaces and without regard to letter case, in any order. |
| Width | Every line must have as many fields as the header (`EH3D-CINT-109`). |

## Columns

Exactly these six columns, each once (a missing, repeated or unknown column is refused: `EH3D-CINT-107`, `EH3D-CINT-108`):

| Column | Required value | What becomes of it |
|---|---|---|
| `reference` | yes | The reference designator (`R1`, `C3`). It is listed in the `references` of the component. Two lines must not use the same reference, whatever the letter case (`EH3D-CINT-111`). |
| `manufacturer` | yes | Part of the identity of the component, as written on the data sheet. |
| `mpn` | yes | The exact part number; the other part of the identity. |
| `value` | no | An observation of the field `value` (`10 kΩ`). |
| `package` | no | An observation of the field `package` (`0603`). |
| `datasheet` | no | An observation of the field `datasheet` (a link or a file name). It is only text: nothing is downloaded and nothing is checked. |

An empty required value is refused (`EH3D-CINT-110`). Surrounding spaces of every value are removed.

## What the import makes

* Lines with the same manufacturer and part number (letter case and surrounding spaces do not matter) become **one component** that lists all its references. Of the spellings of one part the smallest one in byte order is written, so the result does not depend on the order of the lines.
* Every non-empty value becomes an observation with the method `transcribed` and, as its source, the name of the file (the name only, not the folder). The page, table, revision and date are left empty: a person adds them.
* The origin of every component is `import` with the name of the file as its source.
* The licence of the data is **not** invented: it is empty unless the settings file gives `import_licence`. A component without a licence is found by `check` and cannot be exported.
* When two lines give different values for one field of the same part, **both stay** and `check` reports the conflict. Nothing is chosen for the user. Identical values are kept once.
* The status of every component is `draft`.

## Limits

All of them are lines of the settings file (`examples/settings.cfg` shows every key with its default). A file over a limit is refused.

| Key | Default | Refusal |
|---|---|---|
| `csv_max_bytes` | 16777216 | `EH3D-CINT-101` |
| `csv_max_rows` (data lines) | 100000 | `EH3D-CINT-102` |
| `csv_max_columns` (at least 6) | 16 | `EH3D-CINT-103` |
| `csv_max_field_bytes` | 4096 | `EH3D-CINT-104` |
| `csv_max_issues` (problems listed before stopping) | 50 | none |
| `max_components`, `max_observations` | 200000, 64 | `EH3D-CINT-101` |

## Codes

| Code | Meaning |
|---|---|
| `EH3D-CINT-101` | a size limit was exceeded (bytes, components, observations of one field) |
| `EH3D-CINT-102` | more data lines than `csv_max_rows` |
| `EH3D-CINT-103` | a line with more columns than `csv_max_columns` |
| `EH3D-CINT-104` | a field longer than `csv_max_field_bytes` |
| `EH3D-CINT-105` | the file is not valid UTF-8 |
| `EH3D-CINT-106` | a quotation mark used where the format does not allow it, or a bare carriage return |
| `EH3D-CINT-107` | no header, a missing required column or a repeated column |
| `EH3D-CINT-108` | a column that is not one of the six |
| `EH3D-CINT-109` | a line with a different number of fields than the header |
| `EH3D-CINT-110` | an empty `reference`, `manufacturer` or `mpn` |
| `EH3D-CINT-111` | a reference used twice |
| `EH3D-CINT-112` | the output file exists and `--overwrite` was not given |
| `EH3D-CINT-113` | the output file could not be written |

Codes are never reused. A problem is printed as `EH3D-CINT-111 line 3: the reference ...`; the exit code is 2 when the input is unusable.
The findings of `check` on a database keep their own codes, `PHOENIX-CINT-001` to `PHOENIX-CINT-007`.

## Not in this version

Other formats (the files of other design programs, spreadsheets), more columns (quantity, description, alternatives), and merging an import into a database that already exists. See `NOT_YET.md`.
