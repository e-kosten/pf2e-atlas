# Structured Editor SRP Decomposition

Status: done  
Priority: soon  
Owner: unassigned  
Last reviewed: 2026-05-05

## Outcome

The structured search editor action layer is split by durable responsibility. The old broad metadata-action hook was removed instead of retained as a compatibility wrapper.

Current owners:

- `structured-draft-host-mutations.ts` applies bounded host mutations against canonical `SearchRequest` state.
- `structured-draft-grouped-paths.ts` owns pure grouped-field member path calculations.
- `structured-draft-grouped-field.ts` owns grouped field seed state and explorer field-state translation.
- `structured-draft-explorer-actions.ts` owns shared-explorer-backed field flows and prompt-facing explorer adapters.
- `structured-draft-prompt-actions.ts` owns prompt-local clause builders and value-entry flows.
- `structured-draft-structural-actions.ts` owns root, group, leaf, `not`, insertion, move, wrap, unwrap, lift, remove, and toggle routing.
- `structured-draft-entry-actions.ts` owns React composition wiring across the structured-draft action owners.

## Constraints Preserved

- `SearchRequest` remains the canonical long-lived search state.
- Structured-editor child flows apply through bounded host mutations and canonical query-core operations.
- Shared explorer internals remain generic; search-specific continuation and resume behavior stays in the structured search host.
- Group-local resume remains anchored by canonical `groupPath`, and exact-node resume remains a narrow node-scoped exception.
- The removed metadata-action path has no source or test compatibility shim.

## Related

- [TUI architecture](../../../architecture/node/tui.md)
- [Architectural boundaries](../../../architecture/node/boundaries.md)
- [ADR 0013: TUI canonical search state and derived editor projections](../../../architecture/decisions/0013-tui-canonical-search-state-and-derived-editor-projections.md)
- [ADR 0015: shared explorer host contract and live group editing](../../../architecture/decisions/0015-shared-explorer-host-contract-and-live-group-editing.md)
- [Structured editor continuation model convergence](./structured-editor-continuation-model-convergence.md)
