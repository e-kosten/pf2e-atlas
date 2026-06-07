# Rust Web Text-Scoped Filter Counts

Status: deferred
Priority: later
Owner: unassigned
Last reviewed: 2026-06-07

## Problem

The Atlas web prototype exposes dynamic filter discovery and counts, but V1 counts are scoped only by the current `BasicSearchFilter`. In text-search mode, filter counts do not account for the query text or ranked text-result space.

That is acceptable for the first vertical integration because true query-scoped facet counts require a deeper search/discovery design. The app layer should not duplicate text retrieval, fusion, or ranking semantics just to compute counts.

## Desired Outcome

Design and implement text-search-scoped filter discovery/counts for the web search experience.

The design should decide:

- whether counts are computed over the complete ranked candidate set, a bounded result window, or another query-derived eligible-record relation;
- how text-search count semantics should be labelled when they differ from browse/list counts;
- how to keep selected zero-count values visible;
- how to avoid duplicating retrieval/fusion semantics in frontend or app-service code;
- what `atlas-search` API shape should expose query-aware discovery to app-service.

## Constraints

- Keep discovery semantics owned by `atlas-search` and app-service, not frontend code.
- Do not make `atlas-app-service` assemble index internals or bypass runtime/search boundaries.
- Preserve the V1 filter-scoped discovery contract until query-scoped semantics are designed.

## Related

- [Architecture overview](../../architecture/overview.md)
- [ADR 0029: Local web app boundary](../../architecture/decisions/0029-local-web-app-boundary.md)
- [Rust search retrieval and fusion controls](../../architecture/decisions/0024-rust-search-retrieval-and-fusion-controls.md)
