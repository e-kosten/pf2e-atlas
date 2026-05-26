# Rust Artifact Crate Boundary Review

Status: proposed
Priority: later
Owner: unassigned
Last reviewed: 2026-05-26

## Problem

`atlas-artifact` currently owns the physical SQLite artifact contract: table and column descriptors, artifact metadata constants, schema SQL helpers, insert/select SQL helpers, vector blob encoding, and test support.

After retiring the LadybugDB and GraphQLite spike paths, the only production artifact backend is SQLite. Most `atlas-artifact` usage is now inside `atlas-index`, especially the SQLite reader, writer, validation, filter compiler, vector query, and tests. The remaining external uses are small: ingest uses artifact metadata constants for manifest writing, and CLI tests use artifact test support.

This raises a legitimate crate-boundary question: should `atlas-artifact` remain a separate contract crate, or should it become a private SQLite schema module inside `atlas-index`?

## Desired Outcome

Decide whether `atlas-artifact` should remain a standalone crate or be folded into `atlas-index`.

If folded, likely shape:

- Move schema descriptors and SQL helpers to `atlas-index/src/sqlite/schema/`.
- Move metadata constants either to that module or to a narrow public index-owned artifact identity API.
- Move vector blob storage helpers beside SQLite vector query/write code.
- Move test support into `atlas-index` test support and update CLI tests to consume it through `atlas-index`.
- Remove `atlas-artifact` from the workspace.

If retained, document why it is intentionally separate despite being SQLite-specific.

## Constraints

- Do not mix this with Ladybug/GraphQLite cleanup or product feature work.
- Preserve one authoritative table/column contract; do not let reader, writer, and validation code maintain independent schema lists.
- Keep ingest from owning or touching database connections or writer policy.
- Keep any remaining cross-crate artifact identity API narrow and explicit.

## Notes

The current split is coherent: `atlas-artifact` describes what a completed SQLite artifact looks like, while `atlas-index` reads, writes, validates, and queries it. The question is whether that contract boundary is still worth a separate crate now that there is no alternate graph artifact backend.
