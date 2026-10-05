# Observed PF2e Source Schema

`pf2e.json` is the full `pf2e-source-schema/v1` snapshot for manifest-declared pack documents at PF2e commit `4cbdaa37d6c33e9519561bae2c59a23e0288cbce`. It includes embedded data and excludes ingest's `_folders.json` control files. Each JSON path row is formatted on one line to keep data review practical. Examples remain available in fresh discovery reports and are omitted from this baseline to avoid duplicating source prose.

The snapshot records observed paths, JSON types, document/occurrence counts, duplicate object members, with optional source examples. Its signature hashes the actual manifest and selected source bytes/paths. It does not declare model ownership, omission decisions, or every possible upstream schema. Paths are grouped by the root document/type; nested arrays merge member shapes under `[]`, and known keyed maps use `*`.

Generate a candidate from a complete checkout at the intended upstream commit:

```bash
cargo run -p atlas-cli -- index audit-source-paths \
  --source /path/to/pinned-pf2e --json > candidate-schema.json
```

Compare before replacing the checked-in snapshot:

```bash
cargo run -p atlas-cli -- index audit-source-paths \
  --source /path/to/pinned-pf2e \
  --baseline contracts/source-schema/v1/pf2e.json --strict --limit 1
```

CI runs this comparison against the pinned source. Exit 3 means added/removed paths, changed JSON type sets, or changed duplicate-member presence. Counts, examples, and authored value changes do not trigger schema drift. A filtered source subset needs a matching baseline; truncated output cannot be a baseline. See [ADR 0037](../../../docs/architecture/decisions/0037-source-schema-discovery.md).
