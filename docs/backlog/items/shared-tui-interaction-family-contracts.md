# Shared TUI Interaction Family Contracts

Status: proposed  
Priority: later  
Owner: unassigned  
Last reviewed: 2026-04-22

## Problem

The TUI already has strong shared lower-level interaction machinery, but the higher-level behavior shape is still only partially explicit.

The repo has several durable interaction families, yet the intended split between shared behavior contracts and feature-local workflow ownership is not captured in one place. That makes it too easy for future work to centralize the wrong things, leave behavior policy local when it should be shared, or blur unlike interaction families together.

The missing tracked shape is not just "make more helpers shared." It is:

- which interaction families are durable enough to deserve shared behavior contracts
- which user-facing behaviors belong in those shared contracts
- which behaviors must remain feature-local even when the family is shared

## Desired Outcome

Document and implement the intended higher-level interaction-family model for the TUI, with one explicit shared contract shape for each durable family that needs it.

The near-to-medium-term interaction families to track are:

1. `list/detail`
2. `menu/editor/workspace`
3. `command/action-target`

For each family, the repo should make explicit:

- the shared semantic intents the family owns
- the default focus/entry/exit rules the family owns
- the dead-end, no-op, or unavailable-action behavior the family owns
- the parts of data modeling, workflow state, and success-path actions that remain local to screens

This item is intentionally broader than any one family-specific refactor. It tracks the full intended shape of higher-level interaction contracts so future family-specific work lands into a coherent model instead of as isolated follow-ups.

## Intended Family Shape

### 1. List/detail

Shared contract should own:

- rightward intent kinds such as `drill`, `open`, `preview`, or `none`
- dead-end handling for rightward intent
- explicit-only pane-focus policy
- shared notification treatment for lightweight failed navigation

Feature screens should still own:

- row and detail content
- how rows map to destination availability
- success-path reducer actions and async workflow outcomes

Tracked by the narrower follow-up:

- [Shared list/detail behavior contracts](./shared-list-detail-behavior-contracts.md)

### 2. Menu/editor/workspace

Shared contract should own:

- workspace-style selection semantics
- behavior distinctions between local narrowing, content editing, and opening secondary pickers
- coupling between live actions, help/footer text, and visible availability
- common unavailable/no-op behavior where the same editor policy recurs across screens

Feature screens should still own:

- domain data and editor content
- screen-specific workflow state
- actual mutations or workflow actions after an intent resolves

Tracked by the narrower follow-up:

- [Shared menu/editor behavior contracts](./shared-menu-editor-behavior-contracts.md)

### 3. Command/action-target

Shared contract should own:

- explicit entry and exit semantics for command palette and action-target focus
- action-target navigation and apply behavior
- shared help/footer derivation from the active interaction model
- rules for keeping command/action surfaces aligned with actual live availability

Feature screens should still own:

- command definitions and action meanings
- which actions are available in their current workflow state
- screen-specific consequences after a command or target action executes

Existing shared foundation and adjacent cleanup:

- [ADR 0006: Shared TUI interaction contracts](../../architecture/decisions/0006-shared-tui-interaction-contracts.md)
- [Search interaction cleanup](./search-interaction-cleanup.md)

## Constraints

- Do not collapse unlike interaction families into one generic reducer or one generic screen type taxonomy.
- Do not treat lower-level prompt/modal mechanics as the same problem as higher-level interaction-family behavior contracts.
- Do not centralize domain data ownership, workflow state, or success-path business logic that should remain in screens and workflows.
- Prefer explicit family contracts over informal conventions once a family is clearly durable.

## Notes

This item is the umbrella follow-up for the intended higher-level TUI interaction shape.

It exists alongside narrower family-specific items so the repo keeps one tracked statement of:

- which interaction families matter
- what each family should centralize
- what each family should leave local

Prompt/modal concerns are still important, but they are lower-level framework and typing seams rather than the primary higher-level behavior-family leak. Those remain tracked separately through:

- [Typed seams cleanup](./typed-seams-cleanup.md)

This item is also adjacent to the broader boundary-enforcement question:

- [Shared UI model boundary enforcement](./shared-ui-model-boundary-enforcement.md)

## Related

- [TUI architecture](../../architecture/node/tui.md)
- [Architectural boundaries](../../architecture/node/boundaries.md)
- [Shared list/detail behavior contracts](./shared-list-detail-behavior-contracts.md)
- [Shared menu/editor behavior contracts](./shared-menu-editor-behavior-contracts.md)
- [Search interaction cleanup](./search-interaction-cleanup.md)
- [Shared UI model boundary enforcement](./shared-ui-model-boundary-enforcement.md)
