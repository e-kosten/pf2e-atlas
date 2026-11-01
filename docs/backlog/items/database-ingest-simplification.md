# Database and ingest simplification

Status: in_progress
Priority: soon
Owner: database-ingest workstream
Last reviewed: 2026-11-01

## Outcome and evidence

Implementation and local validation are complete for the DB/ingest review scope
selected by the user on 2026-10-31; the single feature PR awaits review. The original
review used clean `64171f6d5b00e7aeec2a815ce103169b46fb90b0`, source tracing and
isolated SQLite probes. It identified an incomplete-source publication risk,
repeated content/numerical representations, saved-domain integrity gaps and
mismatched data lifetimes. The
[dated findings](../../research/database-ingest-simplification.md) record actual
preservation, recovery and query results. Cached release builds used less observed
peak RSS and took longer than the matched baseline.

## Implemented scope

1. Reject publication when declared pack discovery is incomplete; preserve prior output and expose compact admission/exclusion evidence through setup reporting.
2. Enforce closed saved-state enums and encounter/participant membership, with strict decoding and atomic state-preserving migrations. Remove the unused participant-adjustments table only after accounting for historical rows.
3. Separate compact lifetime Next-turn receipts from one current outgoing snapshot; retain receipts across reset/rest. Batch all six roster child collections, including manual trackers, within one snapshot.
4. Store canonical selected content sections and reference them from lexical/semantic occurrences, Core and navigation. Remove JSON identity joins and address reconstruction from chunk coverage.
5. Replace eager previous-corpus hydration with compact inventory/resolver evidence, bounded preparation and component-aware reconciliation. Preserve reference dependency invalidation.
6. Replace per-occurrence owned vectors with exact numerical handles and bounded publication. Give abandoned inference jobs/checkpoints an explicit lifecycle while retaining required retry work.
7. Add index-friendly positive eligibility lowering with exact membership parity and evidence-backed index maintenance.

## Acceptance and boundaries

Keep one SQLite workspace, checked source authority, explicit unavailable states, stable record identities, exact native-input identity, coherent saved/reference transactions and complete validation of the publishing transaction. Existing source/model assets and the installed workspace remain protected. Published migrations are immutable; compatible metadata upgrades preserve saved rows, vector bytes and receipt identities, or fail atomically with actionable diagnostics on invalid historical data. Retired source/preparation contracts follow the existing explicit replacement path while saved-state upgrades remain available.

Validate the incomplete-pack failure path, populated migrations/rollback, source/relationship changes, lexical-only and single/two-profile sections, forced pending-vs-published values, retry/reset lifetimes and nested filter uncertainty. Measure current release builds for preparation, memory, logical writes, maintenance downtime and selective query plans before claiming performance improvements. Each completed refactor removes its replaced production path.

Local integration and measurements use main
`d7187e7834110e3a524e651147ee866b3ad8cd27`, whose published schema 24 includes manual
trackers and the encounter clock. Preserve migrations
23/24 and their countdown conversion; this work adds named steps 25–29 in one PR
and one release slice. Populated 24→29 upgrades must preserve configured round
length, timed condition progress, trackers, baseline/outgoing payloads and all prior
saved identities. Earlier schema-27 task measurements remain historical. Fresh
integrated validation and all eight current corpus-operation closeouts passed, including
24→29 preservation and the final original-source/schema-22 restore followed by a
normal 22→29 metadata upgrade.

The local execution plan retains the ordered blocks, evidence matrix and candidate
lineage. Independent review covers the local candidate; hosted CI covers the
published candidate.
This item stays in progress during PR review; merge and release remain separately
authorized. No further performance optimization is assigned by these observations.

## Related owners

- [Architecture overview](../../architecture/overview.md), [runtime](../../architecture/runtime.md), [artifact contract](../../architecture/artifact-contract.md)
- [Persistent workspace](../../architecture/decisions/0051-persistent-workspace.md), [source-backed reference](../../architecture/decisions/0046-source-backed-reference-artifact.md), [atomic mutable workflows](../../architecture/decisions/0048-atomic-mutable-workflows.md), [encounter recovery](../../architecture/decisions/0061-encounter-recovery.md)
- [Reference-build performance](reference-build-performance.md) retains earlier measurements and completed optimizations. This newly selected scope does not reopen unrelated inference/hardware tuning or repeat historical studies automatically.
- [Validation](../../development/validation.md), [worktree/resource policy](../../development/worktrees-and-cleanup.md)
