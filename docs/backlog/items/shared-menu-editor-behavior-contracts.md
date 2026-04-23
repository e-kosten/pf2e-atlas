# Shared Menu/Editor Behavior Contracts

Status: proposed  
Priority: later  
Owner: unassigned  
Last reviewed: 2026-04-22

## Problem

The TUI already has shared interaction routing, shared prompt adapters, shared help/footer derivation rules, and some shared workspace/query-summary owners, but menu-style editor and workspace surfaces still keep too much behavior policy in feature-local code.

That means the repo has shared mechanics for editor-like flows, but not yet a clearly shared behavior contract for:

- what selection means on workspace-style rows
- when `/` narrows a visible option set versus editing real query content
- how visible actions, command-palette entries, and action availability stay aligned
- when editor-like surfaces should open a secondary picker, act immediately, or no-op
- how menu/editor help and footer behavior should remain coupled to the live interaction model across unlike workspace screens

This is the same general class of abstraction leak as the list/detail behavior problem, but for menu/editor/workspace surfaces rather than two-pane browsing surfaces.

## Desired Outcome

Add shared infrastructure for menu/editor behavior types so editor-like screens map their local data and workflow outcomes into one common behavior contract instead of open-coding behavior policy per screen.

That work should:

- model shared behavior concepts for workspace-style selection and editing without introducing rigid screen classes
- keep help/footer/action availability derived from the same live interaction model
- clarify when local narrowing, content editing, and secondary pickers are the right shared behaviors
- keep domain data, workflow state, and success-path actions in feature screens
- make the higher-level shared editor behavior pathway stable enough to become enforceable later if it proves durable

## Constraints

- Do not flatten unlike editor workflows into one generic reducer.
- Do not treat this as a list/detail extension; it is a separate interaction family.
- Keep prompt/layout mechanics on the existing prompt and modal owners.
- Prefer a shared behavior contract over feature-local conventions when the same policy appears in multiple editor/workspace surfaces.

## Notes

This is adjacent to, but distinct from:

- [Shared list/detail behavior contracts](./shared-list-detail-behavior-contracts.md)
- [Search interaction cleanup](./search-interaction-cleanup.md)
- [Shared UI model boundary enforcement](./shared-ui-model-boundary-enforcement.md)

Prompt seam friction is already tracked separately in [Typed seams cleanup](./typed-seams-cleanup.md); this item is about higher-level editor behavior policy, not prompt result typing.

## Related

- [TUI architecture](../../architecture/tui.md)
- [Architectural boundaries](../../architecture/boundaries.md)
- [ADR 0006: Shared TUI interaction contracts](../../architecture/decisions/0006-shared-tui-interaction-contracts.md)
- [Shared UI model boundary enforcement](./shared-ui-model-boundary-enforcement.md)
