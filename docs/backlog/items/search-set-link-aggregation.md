# Link counts across a filtered record set

Status: proposed
Priority: later
Owner: unassigned
Last reviewed: 2026-11-06

## User task

Answer questions such as: “What equipment do undead creatures of levels 5–7
link to, ranked by how many matching creatures link to each entry?” Allow the
opposite direction as well. This is future work, separate from PR #218's link
picker and shared self-exclusion changes.

## Existing foundation

The connected-picker resolver already compiles parent static eligibility through
ordinary predicate SQL and joins it to resolved relationship occurrences. It
retains saved-list, trait-reference and inherited connected restrictions, direction,
and Main/Whole entry semantics. The current projection returns distinct connected
candidate keys; it does not expose aggregate counts or count-based ordering.

Extend that same indexed relation with grouped aggregates, rather than counting
paged search results, iterating individual records or introducing another filter
evaluator. Parent text/Similar admission remains separate from static eligibility.
See [ADR0057](../../architecture/decisions/0057-relationship-scoped-search.md).

## Proposed outcome

- Choose a statically filtered source set and a target kind/filter, such as
  undead creatures at levels 5–7 and equipment.
- Return connected entries with an explicit count basis and deterministic
  descending count order before pagination, including stable ties and totals.
- Distinguish distinct matching parent records from raw resolved occurrence counts.
  Proposed default: count distinct matching parents so repeated citations from
  one creature do not inflate its rank. Whether to expose both metrics is open.
- Preserve exact direction/owner semantics and attributable evidence, so a count
  can be inspected to see the matching parents and their link occurrences.
- Present results as links/references; an equipment reference does not establish
  that a creature owns, carries or uses that equipment.

## Decisions and validation

Settle count labels, duplicate and embedded-owner handling, and edition preference
before implementation. Parent eligibility currently precedes edition preference;
any candidate edition suppression must keep counts attached to the exact entry
and matching edges, without inventing or merging links across editions.

Use exhaustive static membership rather than the visible page or a bounded graph
preview. Reuse shared predicate/scope resolution, generation checks and coherent
workspace snapshots. Keep SQL aggregation in atlas-index, retrieval policy in
atlas-search and product contracts/projection in app-model/app-service.

Validate both directions, Main/Whole scope, duplicate citations, multiple owners,
static AND/OR/NOT, saved-list/trait scopes, no matches, stable ties, counts beyond
the first page, and evidence explaining every counted parent. Measure a full-corpus
query before introducing caching or a new materialized relationship projection.
