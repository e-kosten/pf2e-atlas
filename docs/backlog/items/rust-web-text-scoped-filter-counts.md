# Rust Web Text-Scoped Filter Counts

Status: deferred
Priority: later
Owner: unassigned
Last reviewed: 2026-10-09

## Problem

Source-backed discovery already shares typed eligibility and text scope with
retrieval. Supported facet contexts freeze the clause-removed candidate universe;
unsupported Boolean contexts return explicit errors. Further UX work should make
these limits and semantic candidate coverage clear without duplicating retrieval
or ranking in the app layer.

## Desired Outcome

Refine text-scoped facet presentation and evaluate broader supported contexts.

Follow-up work should address:

- how to explain complete lexical and bounded semantic candidate universes;
- whether broader Boolean facet contexts are useful enough to support;
- how text-search count semantics should be labelled when they differ from browse/list counts;
- how to keep selected zero-count values visible;
- how to avoid duplicating retrieval/fusion semantics in frontend or app-service code;
- which refinements belong in the existing `atlas-search` discovery API.

## Constraints

- Keep discovery semantics owned by `atlas-search` and app-service, not frontend code.
- Do not make `atlas-app-service` assemble index internals or bypass runtime/search boundaries.
- Preserve shared typed discovery semantics; do not add a second facet compiler.

## Related

- [Architecture overview](../../architecture/overview.md)
- [ADR 0029: Local web app boundary](../../architecture/decisions/0029-local-web-app-boundary.md)
- [Source-backed artifact decision](../../architecture/decisions/0046-source-backed-reference-artifact.md)
