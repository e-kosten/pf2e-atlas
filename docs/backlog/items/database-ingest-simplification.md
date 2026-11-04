# Database and ingest simplification

Status: in_progress
Priority: soon
Owner: database-ingest workstream
Last reviewed: 2026-11-03

[PR184](https://github.com/e-kosten/pf2e-atlas/pull/184) implements bounded source
preparation, input-owned vectors and atomic publication. It also adds canonical
selected sections, indexed eligibility, stricter saved-state constraints and
separate lifetime turn receipts. The upgrade preserves saved state and stable
identities through the coherent schema 24 to 29 migration sequence.

[ADR0065](../../architecture/decisions/0065-input-owned-reference-publication.md)
owns the publication and retrieval decisions. The
[benchmark note](../../research/database-ingest-revision.md) records performance,
preservation evidence and comparison limits. Ordinary ingest validates its write
boundaries. `atlas index validate` provides the separate deep audit.

## Deferred reference rebuild

Consider an explicit command that replaces generated reference data through the
existing publisher while preserving app data and stable identities. This remains
separate from the ingest changes in PR184.
