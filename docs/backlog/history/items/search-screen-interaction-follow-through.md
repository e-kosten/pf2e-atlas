# Search Screen Interaction Follow-Through

Status: done  
Priority: now  
Owner: unassigned  
Last reviewed: 2026-04-21

## Problem

This item is complete. It remains here as durable context for the search-screen interaction cleanup that landed in the split `src/tui/search-screen/` module tree.

The original note preserved useful direction from the dropped `feat/search-final-state-completion` scratch work, which had targeted the old top-level files:

- `src/tui/search-screen-controller.ts`
- `src/tui/search-screen-interactions.ts`
- `src/tui/search-screen-workspace.ts`

That patch is no longer mergeable as a literal diff, but the architecture and product behaviors it described are now present on `main`.

## Desired Outcome

That outcome is now landed:

- keep search-screen input routing on a typed shared interaction seam rather than controller-local raw event handling
- keep help titles, footer bindings, and action availability derived from the same interaction model that executes intents
- preserve the product decision that unavailable setup actions should not appear as inert command-palette entries when the palette is filtered down to commands the user can actually run
- keep the search editor row itself visibly unavailable with a reason when the action is blocked, even if the command palette hides the unavailable command

## Constraints

- Do not resurrect the old top-level `src/tui/search-screen-*.ts` file layout. Any follow-through work should start from the current `src/tui/search-screen/` tree.
- Treat the dropped worktree as design input, not as a patch to replay.
- Keep the search screen aligned with the shared TUI interaction contract rather than reintroducing feature-local input decoding.
- Keep the search screen aligned with the shared list/detail presentation layer where it already applies.
- Avoid broad refactors unless they directly improve the current search-screen interaction model.

## Notes

### Source context

This item preserves the useful intent from the dropped dirty worktree `feat/search-final-state-completion`.

Useful ideas retained from that scratch state:

1. The controller should receive typed search-screen intents and own semantic responses to them, while the router owns terminal event decoding.
2. Search help/footer text should be derived from the same action model used for live routing.
3. The command palette should hide unavailable setup commands rather than showing inert entries that the user cannot execute.

### Why the old implementation was dropped

- `main` already contains a newer split search-screen architecture under `src/tui/search-screen/`.
- `main` already has a `useSearchScreenInteractionRouter` seam and related interaction helpers, so part of the old work is already superseded.
- The old diff overlapped with newer structural work and would have required manual reinterpretation rather than a normal merge.

### Validation focus

Future regressions should be checked against:

- verify that search-screen controller code does not drift back toward direct terminal-event branching
- verify that help/footer/action tables stay generated from one interaction model
- verify command-palette behavior around unavailable setup actions with focused TUI tests
- prefer adding or tightening tests in the current `tests/tui/search-screen.test.tsx` suite rather than reviving older test paths

## Related

- [Backlog: Search interaction cleanup](../../backlog.md)
- [TUI architecture](../../../architecture/node/tui.md)
- [Architectural boundaries](../../../architecture/node/boundaries.md)
- [ADR 0006: Shared TUI interaction contracts](../../../architecture/decisions/0006-shared-tui-interaction-contracts.md)
- [ADR 0009: Shared list/detail presentation layer](../../../architecture/decisions/0009-shared-list-detail-presentation-layer.md)
