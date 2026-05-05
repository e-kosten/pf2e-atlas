# Structured Editor Routing Layer

Status: in_progress  
Priority: now  
Owner: unassigned  
Last reviewed: 2026-05-05

## Problem

Structured query editing can regress when route choice is spread across structural actions, prompt adapters, explorer adapters, and query-state helpers. Grouped same-field edits and single-clause edits need an explicit owner before prompts, shared explorer sessions, or host mutations run.

## Desired Outcome

Structured-draft field editing should have one classifier and one executor:

- grouped-field routes edit same-field cohorts in the containing canonical boolean group
- leaf routes edit one semantic clause
- scope is a root-singleton leaf
- scope changes prune scope-dependent metadata, metric, and action-cost clauses when the category changes
- metrics are ordinary leaf routes whose key discovery may use the shared explorer as a child surface
- `linksTo` and `linkedFrom` are executable record-key leaf routes
- pack, rarity, action cost, and grouped metadata set or discrete fields write through grouped-field replacement
- retired prompt wrappers, exact-node fallback routing, generic serializer fallback, and generic final writeback remain lint-blocked

## Constraints

- Keep `SearchRequest` as the only long-lived query state.
- Keep structured-draft route semantics under `src/tui/search-screen/structured-draft/`.
- Do not move structured-editor writeback semantics into `src/tui/filter-explorer/`.
- Preserve the two semantic edit routes: grouped-field and leaf.

## Related

- [TUI architecture](../../architecture/tui.md)
- [Architectural boundaries](../../architecture/boundaries.md)
- [ADR 0015](../../architecture/decisions/0015-shared-explorer-host-contract-and-live-group-editing.md)
- [Search interaction cleanup](./search-interaction-cleanup.md)
