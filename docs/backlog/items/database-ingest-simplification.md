# Database and ingest simplification

Status: in_progress
Priority: soon
Owner: database-ingest workstream
Last reviewed: 2026-11-01

## Outcome and evidence

An initial implementation and validation candidate exists in
[PR184](https://github.com/e-kosten/pf2e-atlas/pull/184). The user requested an
architectural revision before merge after cached rebuild time rose from about
242 to 518 seconds despite lower peak memory. Passing candidate checks do not
establish acceptance of that regression or of the revised design. The original
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
This item stays in progress during architecture research and revision; merge and
release remain separately authorized. Earlier validation applies to its recorded
candidate and must not be presented as validation of the revised design.

## Architecture revision and next decision

Accepted intended changes, pending implementation:

- Publish captured inputs. Check each external file, manifest and catalog against
  capture when consuming it, then own the data; later changes belong to the next
  build. Remove the final freshness rescan. Missing/unreadable declared packs
  still fail discovery; deliberate removal and existing empty packs remain valid.
- Bind context, generation, prepared records and selected numerical data in one
  owned publication. Keep bounded decoding and consumed-data corruption checks;
  remove repeated whole-build authentication between private stages.
- Prepare related rich metadata in bounded slices, retaining compact global
  relation knowledge and efficient model batches. Hold only a bounded inference
  result batch in RAM, cache/stage it and release it. Two temporary disk copies of
  new inference for recovery and publication are acceptable.
- Retain one recovery result per exact model/execution/input key for unpublished
  inference; reuse published values separately. Preserve cache results after
  failure, allow simple oldest-first eviction and clear the cache on successful
  complete publication. Selected publication data must survive cache eviction.
  Remove build-target history maintained solely to make early eviction safe.
- Group staging under one owned attempt directory, clean normal completion/error
  and safely remove abandoned attempts using update ownership. No staging quota
  manager or per-record reservation ledger has been agreed.
- Setup/upgrades may be maintenance windows. Preserve atomic publication, saved
  state and complete validation of every real publication. No unchanged-update,
  forced-build or deep-audit semantics change is implicitly accepted.

The [numerical ownership study](../../research/embedding-value-ownership.md)
measured 0.629% duplicate document excess and completed 192 paired SQL cases.
The occurrence-capped input-index comparison preserved eligible counts/distances,
with three
same-record cutoff-witness changes, but added roughly 10.4–13.5% to the equal-case
warm SQL aggregates. The leading recommendation is stable input ownership with
an explicit occurrence cap, for ingest lifetime simplification rather than query
speed. One active input row with atomic forced replacement needs no permanent
version history. This is a proposal awaiting user design input; the conservative
alternative retains occurrence indexing and one owned transient value copy.

Next: choose that ownership/query tradeoff, then implement the accepted pipeline
simplifications and selected numerical layout together. Validate real migration,
forced numerical dependency invalidation, trait lifetimes, rollback/crash recovery,
saved state and cache/scratch cleanup. Measure matched completed rebuilds and
product queries; the isolated prototype does not establish ingest speed, physical
migration peaks or product relevance. Keep one PR184; refresh final local/CI
validation for the actual revised candidate.

The local plan retains implementation ordering and validation obligations. Cache
thresholds and slice sizes can follow measured resource use; they do not block the
numerical-ownership decision. No revised product implementation is claimed yet.

## Related owners

- [Architecture overview](../../architecture/overview.md), [runtime](../../architecture/runtime.md), [artifact contract](../../architecture/artifact-contract.md)
- [Persistent workspace](../../architecture/decisions/0051-persistent-workspace.md), [source-backed reference](../../architecture/decisions/0046-source-backed-reference-artifact.md), [atomic mutable workflows](../../architecture/decisions/0048-atomic-mutable-workflows.md), [encounter recovery](../../architecture/decisions/0061-encounter-recovery.md)
- [Reference-build performance](reference-build-performance.md) retains earlier measurements and completed optimizations. This newly selected scope does not reopen unrelated inference/hardware tuning or repeat historical studies automatically.
- [Validation](../../development/validation.md), [worktree/resource policy](../../development/worktrees-and-cleanup.md)
