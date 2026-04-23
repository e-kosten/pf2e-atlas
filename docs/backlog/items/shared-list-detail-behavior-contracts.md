# Shared List/Detail Behavior Contracts

Status: done  
Priority: soon  
Owner: unassigned  
Last reviewed: 2026-04-22

## Problem

This item is complete. The shared list/detail layer now includes a dedicated behavior contract owner alongside the presentation seam, and the qualifying callers route rightward list behavior through that shared contract instead of open-coding dead-end policy in feature controllers.

## Desired Outcome

That landed outcome is:

- qualifying callers declare rightward behavior through shared `drill`, `open`, `preview`, or `none` contracts
- callers map local row state into shared destination availability and success-path callbacks
- shared dead-end fallback behavior owns notification-vs-noop policy
- pane focus stays explicit-only for screens that fit the contract
- preview-already-visible behavior is treated as a shared dead end instead of a screen-local special case

## Constraints

- Do not introduce hard-coded shared screen categories like "result screen" or "explorer screen" just to fit current callers.
- Keep data ownership, content building, and success-path workflow mapping in the feature screens.
- Keep lightweight dead-end feedback on the shared list/detail notification seam rather than adding new screen-local footer state.
- Prefer a shared behavior contract that can be reused by search and explorer screens without flattening unlike workflows into one reducer.

## Notes

This was a follow-through item from the shared list/detail presentation work and the right-navigation feedback fixes. The behavior contract now covers:

- filter explorer inspect mode
- filter explorer compose mode
- search result reader

The derived-tag review screen still uses the shared presentation seam but does not fit this specific behavior contract because its rightward behavior is centered on an action-target flow instead of list-row confirm semantics.

This item remains adjacent to the broader [Shared UI model boundary enforcement](./shared-ui-model-boundary-enforcement.md) follow-up, but it landed as a narrower behavior-specific contract rather than a broader reducer taxonomy.

## Related

- [TUI architecture](../../architecture/tui.md)
- [Architectural boundaries](../../architecture/boundaries.md)
- [Right-navigation feedback](./right-navigation-feedback.md)
- [Shared UI model boundary enforcement](./shared-ui-model-boundary-enforcement.md)
