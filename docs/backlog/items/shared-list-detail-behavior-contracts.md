# Shared List/Detail Behavior Contracts

Status: proposed  
Priority: soon  
Owner: unassigned  
Last reviewed: 2026-04-22

## Problem

The shared list/detail presentation layer already centralizes mechanics such as pane sizing, detail slicing, interaction-context setup, and transient footer notifications, but it still leaves semantic behavior policy in feature-local screen code.

That means shared list/detail screens still decide locally:

- what rightward input means for the active list
- whether the focused row has a valid destination
- whether dead ends should notify or silently no-op
- whether pane focus changes are explicit-only or tied to another action

The repo has a shared mechanism, but not yet a shared behavior contract.

## Desired Outcome

Add shared infrastructure for list/detail behavior types so screens map local data and workflow outcomes into one common semantic contract instead of open-coding behavior policy.

That work should:

- model reusable behavior capabilities rather than rigid screen classes
- let screens declare rightward behavior such as `drill`, `open`, `preview`, or `none`
- let screens map focused entries into shared concepts like "has destination" or "dead end"
- centralize dead-end fallback behavior, especially notification vs no-op decisions
- preserve explicit pane-focus actions as a separate concern from rightward list behavior

## Constraints

- Do not introduce hard-coded shared screen categories like "result screen" or "explorer screen" just to fit current callers.
- Keep data ownership, content building, and success-path workflow mapping in the feature screens.
- Keep lightweight dead-end feedback on the shared list/detail notification seam rather than adding new screen-local footer state.
- Prefer a shared behavior contract that can be reused by search, explorer, and review screens without flattening unlike workflows into one reducer.

## Notes

This is a follow-through item from the shared list/detail presentation work and the right-navigation feedback fixes. The current smell is not missing shared mechanics; it is that semantic list/detail behavior is still partly feature-owned.

This item is also adjacent to the broader [Shared UI model boundary enforcement](./shared-ui-model-boundary-enforcement.md) follow-up, but it is narrower and more behavior-specific.

## Related

- [TUI architecture](../../architecture/tui.md)
- [Architectural boundaries](../../architecture/boundaries.md)
- [Right-navigation feedback](./right-navigation-feedback.md)
- [Shared UI model boundary enforcement](./shared-ui-model-boundary-enforcement.md)
