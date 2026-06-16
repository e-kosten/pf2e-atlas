# Rust Local-State Import And Export

Status: proposed
Priority: later
Owner: unassigned
Last reviewed: 2026-06-16

## Problem

Saved lists and future local-state features should live outside the generated SQLite artifact so they survive index rebuilds. That keeps the artifact rebuildable and read-only for runtime retrieval, but it also means users and agents need a deliberate way to back up, move, inspect, and restore durable local state without copying opaque SQLite files by hand.

The first saved-list implementation should keep import/export out of scope so the local-state database, CLI CRUD, and rebuild-survival behavior can land cleanly. This backlog item preserves the follow-up once the local-state schema has at least one real user-facing collection surface.

## Desired Outcome

Add stable import/export commands for local-state data, starting with saved lists.

The design should answer:

- which local-state entities are exported in v1
- whether export is whole-database, selected-list, or both
- how unresolved saved-list item record keys are represented
- how imports handle existing list slugs, duplicate items, and ordering
- whether imports validate record keys against the active artifact or preserve unresolved rows with warnings
- how future local-state entities can join the format without breaking older exports

## Constraints

- Do not put local-state data into the generated artifact as part of export.
- Do not make artifact rebuilds responsible for preserving local-state rows.
- Keep the export format scriptable and agent-editable.
- Preserve unresolved saved-list items rather than silently dropping them.
- Keep import conflict behavior explicit; avoid implicit overwrites.

## Candidate Shape

Likely CLI surfaces:

```text
atlas lists export <slug> --output list.json
atlas lists import list.json
atlas local-state export --output atlas-local-state.json
atlas local-state import atlas-local-state.json
```

The first useful format can be JSON with a top-level format version, exported timestamp, and entity arrays. Saved-list items should include record keys, position, optional notes, and enough unresolved-item metadata to round-trip stale references without requiring the active artifact to contain every key.

## Related

- [Rust web record detail polish](./rust-web-record-detail-polish.md)
