# ADR 0005: Live Search-Semantics Exploration

- Status: Accepted
- Date: 2026-04-20

## Context

The TUI had drifted into treating ontology browsing, search semantics, and facet/query-field picking as adjacent but separate tools:

- ontology browsing exposed the hierarchy and descriptions
- search semantics exposed parts of the query space, but not always the full live branch structure
- query-field picking could collapse back into scoped one-off picker models
- concrete leaves sometimes showed hardcoded samples instead of opening the same result behavior used elsewhere

That split created two recurring problems:

- users could not reliably inspect the full live search space from one exploration model
- branch/value inspection and query building could drift away from the actual browse surface

The agreed end state from the TUI architecture closure work was to treat these surfaces as one exploration problem: browse the full search space, inspect what a branch or value means, see where it applies in the live corpus, and launch real results from that same model.

## Decision

Use the search-semantics ontology as the shared live exploration surface for browse, inspection, and query-field picking.

In practice this means:

- `src/app/ontology-service.ts` continues to publish the full readonly search-semantics browse model
- scoped query-field entry in the TUI should seed the shared explorer directly instead of maintaining a separate reduced picker ontology or hosted compatibility layer
- structured-editor discovery steps that need live field values or metric keys should compose that same shared explorer surface rather than introducing picker-local discovery paths
- concrete search-semantics leaves should launch the normal result behavior against the live corpus instead of showing special-case tiny samples
- scoped entry flows may seed explorer state, but they should still land inside the same underlying browse surface

## Consequences

- Users can inspect the same hierarchy for ontology browsing, search-semantics understanding, and structured query construction.
- Search-semantics branches and leaf results stay aligned with the live corpus instead of drifting toward curated examples.
- Query-field entry stays on the shared explorer surface instead of preserving a parallel picker compatibility stack.
- Structured-editor flows such as pack selection and metric-key selection stay on the same explorer-backed discovery model as the rest of live query-field entry.
- Future work that needs a scoped search-semantics entry point should compose shared explorer state directly, not create another standalone picker tree.
