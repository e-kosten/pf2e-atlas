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
6. Store native vectors by exact input identity and publish through one owned preparation/selection object. Keep unpublished inference in a bounded disposable recovery cache; stage new/forced values and retain useful retry work without target history.
7. Add index-friendly positive eligibility lowering with exact membership parity and evidence-backed index maintenance.

## Acceptance and boundaries

Keep one SQLite workspace, checked source authority, explicit unavailable states, stable record identities, exact native-input identity, coherent saved/reference transactions and checks at the owning construction, consumption and SQL-write boundaries. Whole-reference health checking remains an explicit deep audit; ordinary publication trusts untouched committed data. Existing source/model assets and the installed workspace remain protected. Published migrations are immutable; compatible metadata upgrades preserve saved rows, vector bytes and receipt identities, or fail atomically with actionable diagnostics on invalid historical data. Retired source/preparation contracts follow the existing explicit replacement path while saved-state upgrades remain available.

Validate the incomplete-pack failure path, populated migrations/rollback, source/relationship changes, lexical-only and single/two-profile sections, forced pending-vs-published values, retry/reset lifetimes and nested filter uncertainty. Measure current release builds for preparation, memory, logical writes, maintenance downtime and selective query plans before claiming performance improvements. Each completed refactor removes its replaced production path.

The pre-revision measurements used main
`d7187e7834110e3a524e651147ee866b3ad8cd27`, whose published schema 24 includes manual
trackers and the encounter clock. The revised branch integrated main `f76232c8`
before implementation. Preserve released migrations 23/24 and their countdown
conversion; the final suffix adds named steps 25–29 in one PR and one release
slice. Populated 24→29 upgrades must preserve configured round length, timed condition
progress, trackers, baseline/outgoing payloads and saved identities.

Earlier schema-27 task measurements remain historical. The pre-revision candidate
passed its then-current integrated validation and
eight corpus-operation closeouts, including 24→29 preservation and the final
original-source/schema 22 restore followed by a normal 22→29 metadata upgrade.
Those results describe that earlier candidate and migration suffix; revised
release evidence is collected separately.

The local execution plan retains the ordered blocks, evidence matrix and candidate
lineage. Independent review covers the local candidate; hosted CI covers the
published candidate.
This item stays in progress through revised implementation validation and delivery;
merge and release remain separately authorized. Earlier validation applies to its recorded
candidate and must not be presented as validation of the revised design.

## Architecture revision

The revised implementation follows these accepted decisions:

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
- Setup/upgrades may be maintenance windows. Preserve atomic publication and saved
  state. The subsequent validation refinement below supersedes the earlier
  complete-final-audit requirement for ordinary source publication; forced-build
  and explicit deep-audit meanings remain unchanged.

The [numerical ownership study](../../research/embedding-value-ownership.md)
measured 0.629% duplicate document excess and completed 192 paired SQL cases.
Occurrence-capped input lookup preserved eligible counts/distances, with three
same-record cutoff-witness changes, but added 10.4–13.5% to the equal-case warm
SQL aggregates. The selected design uses stable input ownership and an explicit
occurrence cap for lifetime simplification. One active value with atomic forced
replacement requires no permanent version history; see
[ADR0065](../../architecture/decisions/0065-input-owned-reference-publication.md).

The revised implementation is complete locally. Preparation uses one indexed raw
admission stream, bounded combined slices and a narrow alias-journal pass. Full
model keys bind registry IDs only within admitted sessions. Native validation
streams input-ordered lookups, avoiding a sort carrying vector BLOBs; sparse
maintenance uses public vec0 operations. Replaced target-history, reused-vector
freezing and final source-rescan paths have been removed.

The controlled main24 upgrade preserved saved state, native bytes, consumers and
the released migration prefix. Eight focused historical-admission tests, including four 19→20-only
tests, also preserve the main-era migration20 admission contract. The current full Rust gate passes, as do
unchanged-input frontend, developer, source-generation and release checks, five
explicit real-model fixtures and affected browser/interaction review.

The measured original-source cached rebuild took 292.92 seconds and 0.83 GiB
peak RSS, with zero inference and 323,022 reused inputs; exact metadata and
independent native comparison passed. The later revised executable's cached
retry after the Heal edit and checked recovery completed in 355.95 seconds and
0.86 GiB, again with zero inference and the same reuse count. Both observations
are below the earlier PR's 518.44 seconds; the later retry is above the fresh
main control's 319.59 seconds and historical main's 241.94 seconds. These are
single-host observations with different recorded build/source states and
uncontrolled caches. They establish no consistent speedup over main. Ownership,
API/cache/capture, preparation and validation changes are combined at their
recorded checkpoints, so individual causal savings are not established.

The exact changed-record, real interruption/checked recovery, cached retry and
44 product reads passed. The product reads preserved database bytes, saved state
and shared model assets. See the [revision findings](../../research/database-ingest-revision.md)
for the implemented architecture, measured observations, proof dependencies and
limits. The local plan retains exact artifact identities and validation lineage.

Continue review in PR184; its published head and hosted checks identify the
candidate under review. Local checks, hosted CI and human acceptance of the
performance tradeoff are separate outcomes. Merge and release require explicit
authorization.

## Subsequent ingest and validation simplification

The follow-up removes the corpus-wide native prevalidation/descriptor protocol,
repeated private-frame authentication and ordinary whole-reference final audit.
Bounded native batches prepare once, preserve all occurrences and trait repeats,
and keep one model runner per profile. Compact record sealing and one derived
bundle feed publication; unchanged sections, traits and Core skip reconciliation
with their dependency and pruning rules preserved. Setup carries one owned source
capture into the selected source-build route instead of capturing again.

Ordinary source publication uses one transaction under exclusive admission without
a separate durable quarantine stamp. Failure restores the previous committed
state, including any unrelated pre-existing defect. Existing upgrade, reclamation
and legacy maintenance paths remain. Inference checks its basic numerical contract
once; private staging keeps structural bounds; durable cache/repack bytes retain
checksum checks. Complete reference audit remains `atlas index validate`. Tests
of deliberately malformed internal fixture outputs exercise that explicit audit,
while producer tests protect the normal generation loops.

Implementation, independent review, full local validation and matched final
measurements are complete at `ffe1b50f`. Cached ingest took 141.99 seconds and the
same Heal edit took 144.68 seconds; separate complete audits passed afterward.
The [dated R8 findings](../../research/database-ingest-revision.md#2026-11-02-r8-ingest-and-validation-simplification)
record baselines, assurance changes and measurement limits. The mechanical-only
checkpoint showed no measured speedup. Published-head CI and human acceptance
remain separate delivery gates.

### Deferred reference rebuild

The user deferred an explicit clear-and-rebuild reference operation from this PR.
Future work may rebuild generated reference data while preserving app data and
stable identities, using the existing publisher. This task adds no repair/rebuild
command, option, mode or prerequisite.

## Related owners

- [Architecture overview](../../architecture/overview.md), [runtime](../../architecture/runtime.md), [artifact contract](../../architecture/artifact-contract.md)
- [Persistent workspace](../../architecture/decisions/0051-persistent-workspace.md), [source-backed reference](../../architecture/decisions/0046-source-backed-reference-artifact.md), [atomic mutable workflows](../../architecture/decisions/0048-atomic-mutable-workflows.md), [encounter recovery](../../architecture/decisions/0061-encounter-recovery.md)
- [Reference-build performance](reference-build-performance.md) retains earlier measurements and completed optimizations. This newly selected scope does not reopen unrelated inference/hardware tuning or repeat historical studies automatically.
- [Validation](../../development/validation.md), [worktree/resource policy](../../development/worktrees-and-cleanup.md)
