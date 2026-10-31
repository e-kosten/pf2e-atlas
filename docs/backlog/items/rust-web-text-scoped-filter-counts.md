# Summaries of retrieved semantic candidates

Status: deferred
Priority: later
Owner: unassigned
Last reviewed: 2026-10-30

Ordinary filter discovery uses structured eligibility and saved-list scope.
Semantic ranking selects bounded retrieval candidates without establishing an
exhaustive relevance-qualified membership set. A candidate cap is not a predicate.
Do not expose candidate-window counts through ordinary filter counts or change
browser vocabulary behavior to create a consumer for cached neighbor selection.

Reconsider only for a demonstrated user task asking about the retrieved candidates
rather than all eligible records. Define the exact population, query/profile/
recipe and generation identity, cap/coverage, filter replacement and remaster
behavior, and labels explaining that zero does not establish corpus absence.
A candidate summary must remain distinguishable from exhaustive structured counts.

The local discovery-neighbor experiment was retired. No accepted latency evidence
or current browser consumer justified keeping that path. This deferral does not
change ordinary TextSearch or decide broader Similar/scoped-search reuse.

Related: [search/filter separation](../../architecture/decisions/0011-canonical-search-request-and-filter-tree-model.md).
