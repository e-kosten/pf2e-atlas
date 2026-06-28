# Rust Local-State Import And Export

Status: proposed
Priority: later
Owner: unassigned
Last reviewed: 2026-06-28

## Problem

Saved lists and future local-state features should live outside the generated SQLite artifact so they survive index rebuilds. That keeps the artifact rebuildable and read-only for runtime retrieval, but it also means users and agents need a deliberate way to back up, move, inspect, and restore durable local state without copying opaque SQLite files by hand.

The first saved-list implementation should keep import/export out of scope so the local-state database, CLI CRUD, and rebuild-survival behavior can land cleanly. This backlog item preserves the follow-up once the local-state schema has at least one real user-facing collection surface.

## Desired Outcome

Add stable import/export commands for local-state data. Saved-list JSON import/export now exists; broader local-state export remains future work.

The design should answer:

- which local-state entities are exported in v1
- whether export is whole-database, selected-list, or both
- how unresolved saved-list item record keys are represented
- how imports handle existing list ids, duplicate items, and ordering
- whether imports validate record keys against the active artifact or preserve unresolved rows with warnings
- how future local-state entities can join the format without breaking older exports

## Current Saved-List Surface

Saved lists now support a first portable JSON document shape:

```text
atlas lists export <id>
atlas lists export <id> --output list.json
atlas lists import list.json
atlas lists import list.json --id new-id
atlas lists import list.json --id existing-id --replace
```

`atlas lists export` writes the raw export document, not the standard command JSON envelope, so it can be redirected directly into a file. Imports use the final target id for conflict handling: by default an existing target fails with `saved_list_already_exists`; `--replace` replaces that target list. Exported items include position, record key, resolved record name, status, optional note, and a snapshot so unresolved rows can round-trip.

## Constraints

- Do not put local-state data into the generated artifact as part of export.
- Do not make artifact rebuilds responsible for preserving local-state rows.
- Keep the export format scriptable and agent-editable.
- Preserve unresolved saved-list items rather than silently dropping them.
- Keep import conflict behavior explicit; avoid implicit overwrites.

## Remaining Candidate Shape

Remaining local-state surfaces:

```text
atlas local-state export --output atlas-local-state.json
atlas local-state import atlas-local-state.json
```

Whole-local-state export should reuse the same durable principles as saved-list export: JSON with a top-level format version, exported timestamp, entity sections, explicit conflict handling, and unresolved reference preservation. TSV saved-list import remains a possible convenience format after the JSON contract has more use.

## Related

- [Rust web record detail polish](./rust-web-record-detail-polish.md)
