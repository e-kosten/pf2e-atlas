# Broader exact neighbor reuse

Status: proposed
Priority: soon
Owner: unassigned
Last reviewed: 2026-10-30

## Scope and evidence

Explore exact native-neighbor reuse beyond ordinary unscoped retrieval without
changing retrieval quality, public request state or cancellation. The audit used
merged revision `07ca075467c318fbf5d134555bbea7070c7397b2` from
[PR164](https://github.com/e-kosten/pf2e-atlas/pull/164).
[ADR0062](../../architecture/decisions/0062-exact-text-neighbor-reuse.md) defines
the ownership, identity, bounds and publication contract. The shared ordinary
producer now covers unscoped TextSearch and baseline record/trait Similar.
Measurements from one consumer do not establish benefits for another.

## Remaining candidates

| Consumer | Native work and required extension |
| --- | --- |
| Trait-reference text search | Hybrid retrieval inside referenced record keys. Bind the actual eligible set, including empty scope, rather than the trait label. |
| Configured content/part Similar | Distinct 256-neighbor selections with comparison-role and identity eligibility. Requires matching selection recipe and hydration checks; candidate-vector reads and scoring remain separate. |
| Saved-list text filtering | FTS-only today; neighbor reuse removes no work. |

Confirm concrete first-party repetition before choosing another slice. Trait
Similar defaults to Baseline with separate preview/full-search requests; record
Similar defaults to Passages and benefits from ordinary retention only when
Baseline is selected. Sharing requires identical filters and other native inputs.
A preview reopen can already hit the browser cache. Do not extrapolate baseline
gains to advanced strategies.

Configured Similar can generate up to 256 distinct windows against the current
64-entry store, so repeated requests can churn it. Measure generation, vector
reads and scoring separately, plus mixed-consumer hits, evictions and memory,
before choosing admission or capacity changes.

## Deferred discovery experiment

The local discovery-reuse experiment (`ffc1e77d`) is retired and must not be
integrated. The browser does not request text-context facets; the CLI can repeat
counts and values in one command, but counting a ranked candidate set is a
separate product capability from ordinary filter counts. Ordinary discovery is
to use structured eligibility. Summaries of semantic retrieval candidates are
deferred until a demonstrated use case and explicit product contract exist.
No accepted latency result was obtained for the retired experiment.

## Validation for an extension

- Prove cold/hit equality of deterministic output, coverage, distance bits,
  witnesses and failures for both profiles; distinguish changing telemetry.
- Preserve native exact-k membership and cutoff ties. Filtering global top-k
  cannot produce scoped top-k. Exercise changed/reordered/duplicate/empty scopes,
  exclusions, predicates, Core/All, seed failures and role/identity masks.
- Preserve current-snapshot hydration/admission, generation/publication retirement,
  cancellation, permits, work budgets, bounded allocation and oversized-key bypass.
  Keep ranking, remaster policy and Similar scoring with their existing owners.
- Use matched release operations on the same private corpus/model snapshot and
  serialize builds and timings. Report exact commits, output parity, cold/warm
  latency, native/inference counts, mixed working sets and accounting limits.

Keep policy in shared retrieval/native-candidate owners. Reconcile dedicated
opt-in paths and ADR0062 exclusions when coverage expands; do not add one cache
per surface or retain transitional implementations.

Concurrent-miss sharing is a separate proposed optimization. Measure duplicate
in-flight work before defining leader cancellation, surviving waiters, error
fan-out, admission and permit ownership. No broader speedup is established.
