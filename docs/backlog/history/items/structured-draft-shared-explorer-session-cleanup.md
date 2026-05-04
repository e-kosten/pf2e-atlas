# Structured Draft Shared Explorer Session Cleanup

Status: done  
Priority: later  
Owner: unassigned  
Last reviewed: 2026-05-03

## Problem

This item is complete. It remains here as durable context for the structured-draft metadata action cleanup that landed on 2026-05-03. The broader structured-editor continuation convergence later replaced that intermediate lifecycle coordinator with a search-host continuation model.

Before that cleanup, the search structured-draft metadata actions carried duplicated shared-explorer session wiring across:

- single-clause field editing
- grouped `Add Clause -> Metadata` field editing

Those flows intentionally had different writeback shapes, but they each hand-wired their own explorer lifecycle behavior. That duplication increased drift risk around:

- back and cancel behavior
- apply-latest-on-exit behavior
- seed-state and preserved-field-state handling
- future bug fixes landing in only one launcher path

## Desired Outcome

This outcome is now landed.

The cleanup outcome was an intermediate session coordinator that:

- seeds the explorer from the caller's query and current field state
- normalizes `applied`, `back`, and `cancelled` outcomes
- centralizes explorer exit and cancel handling

The cleanup preserved the caller-owned writeback split:

- single-clause flows still return canonical filter nodes
- grouped-field flows still rebuild grouped replacement nodes and focus paths

## Constraints

- Do not introduce a second long-lived editor or metadata state model.
- Do not collapse grouped-field and single-clause writeback into one generic mutation shape.
- Keep prompt/layout mechanics on the existing framework owners.
- Keep this as a bounded maintainability cleanup rather than a design change to picker or explorer behavior.

## Notes

### Landed seam

The shared explorer lifecycle routed through one file-local coordinator in `structured-draft-metadata-actions.ts`.

The affected callers shared that seam instead of each carrying partially duplicated lifecycle wiring.

### Preserved behavior

The cleanup intentionally keeps the behavioral split between:

- grouped add-clause metadata flows that seed from and write back to the current boolean group
- single-clause edit flows that continue returning canonical nodes into the staged editor loop

The goal was to remove lifecycle drift, not to unify those mutation models.

### Validation that landed with the cleanup

- focused search-screen coverage for grouped metadata-explorer back-navigation
- focused search-screen coverage for single-clause shared-explorer back-navigation
- full `npm run build`
- full `cd scripts && npm test`

## Related

- [Shared menu/editor behavior contracts](./shared-menu-editor-behavior-contracts.md)
- [Search filter explorer API simplification](./search-filter-explorer-api-simplification.md)
- [TUI architecture](../../../architecture/tui.md)
- [Architectural boundaries](../../../architecture/boundaries.md)
