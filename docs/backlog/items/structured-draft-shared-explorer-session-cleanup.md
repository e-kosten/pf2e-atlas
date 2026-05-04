# Structured Draft Shared Explorer Session Cleanup

Status: proposed  
Priority: later  
Owner: unassigned  
Last reviewed: 2026-05-03

## Problem

The search structured-draft metadata actions still carry duplicated session-orchestration logic for opening the shared ontology/filter explorer from:

- single-clause field editing
- grouped `Add Clause -> Metadata` field editing

Those flows legitimately have different writeback semantics, but they should not each hand-wire their own explorer lifecycle behavior. Leaving them duplicated increases drift risk around:

- back and cancel behavior
- `applyLatestOnClose`
- seed-state and preserved-field-state handling
- future bug fixes landing in one launcher path but not the other

## Desired Outcome

Keep the current user-facing grouped and single-clause behaviors, but route both through one explicit shared session coordinator for shared-explorer field launches.

That cleanup should:

- centralize shared explorer session lifecycle handling
- preserve distinct caller-owned writeback behavior
- remove duplicated launch/result wiring from `structured-draft-metadata-actions`
- leave no compatibility-only parallel orchestration path behind

## Constraints

- Do not introduce a second long-lived editor or metadata state model.
- Do not collapse grouped-field and single-clause writeback into one generic mutation shape.
- Keep prompt/layout mechanics on the existing framework owners.
- Treat this as a bounded maintainability cleanup unless implementation discovers a broader shared editor-behavior problem.

## Related

- [TUI architecture](../../architecture/tui.md)
- [Architectural boundaries](../../architecture/boundaries.md)
- [Shared menu/editor behavior contracts](./shared-menu-editor-behavior-contracts.md)
- [Search filter explorer API simplification](../history/items/search-filter-explorer-api-simplification.md)
