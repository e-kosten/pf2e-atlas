# Embedded Document Failure Isolation

Status: deferred
Priority: later
Owner: unassigned
Last reviewed: 2026-10-27

## Problem and desired outcome

A structurally malformed embedded document can make its entire collection
unavailable to typed consumers. For example, a journal's pages containing a valid
text page and a scalar `42` leaves both unavailable for ordinary child reading
and search. Original source remains retained. Invalid fields inside an otherwise
supported child can already be isolated; this task concerns failures at the
child document boundary.

Allow healthy embedded documents to remain usable beside explicit unavailable
entries, preserving authored positions, identity and evidence of incompleteness.
The abandoned H8 pathway demonstrated this resilience for journal pages and
roll-table results. Recover the capability within the current generated source
model rather than adopting the old canonical implementation.

## Scope and constraints

Select the embedded-document collections to support before implementation.
Journal pages and roll-table results are a bounded initial scope; actor items
and recursive physical subitems require their own consumer checks. This is not
permission to weaken atomic admission of predicates, rules or other collections.

- Represent unavailable children through the typed model with retained source
  and diagnostics. Do not salvage raw neighbors in display or indexing code.
- Preserve original positions and stable/snapshot-local addressing, including
  duplicate or unavailable IDs. Never silently drop or renumber a child.
- Carry availability through snapshots, traversal, content, relationships,
  indexing and readers. Usable neighbors must not imply a complete or empty
  collection. Table-roll eligibility and incomplete-result feedback must account
  for unavailable entries explicitly.
- Review generated contracts, snapshot compatibility and required republication.
  Update admission/traversal architecture guidance with the implementation.

## Validation

Cover a valid child before and after malformed entries, multiple failures,
duplicate/missing IDs, and nested collections. Prove authored order, checked
navigation, snapshot round trips, healthy-child content/search and explicit
incompleteness through production consumers. Preserve strict parsing behavior
and existing atomic rule/predicate admission.

No affected record in the currently supported corpus has been established by
the parity audit. Scheduling and implementation design remain deferred.

## Related

- [Source admission and field retention](../../architecture/decisions/0041-source-admission-and-field-retention.md)
- [Source-backed traversal and preparation](../../architecture/decisions/0045-source-backed-record-enrichment.md)
- [Capability recovery tracker](rust-integration-recovery.md)
