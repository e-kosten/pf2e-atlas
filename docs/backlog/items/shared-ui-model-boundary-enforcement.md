# Shared UI Model Boundary Enforcement

Status: proposed  
Priority: later  
Owner: unassigned  
Last reviewed: 2026-04-21

## Problem

The TUI already has centralized UI model and state helpers, but the higher-level shared pathways are not consistently mandatory yet.

The original scratch note preserved this shape:

- shared helpers already exist for app routing, two-pane behavior, action-target behavior, and ontology browsing
- lower-level TUI boundaries are already lint-enforced
- higher-level reusable UI model modules are still partly convention-driven, especially for screen-specific reducers and view-model logic

That means the repo has centralized pieces, but not yet a clearly enforced presentation-model layer.

## Desired Outcome

Identify which reusable UI-model and screen-state abstractions are stable enough to become mandatory, then add enforcement once those shared pathways are ready.

That work should:

- lift repeated screen behaviors into explicit reusable modules first
- clarify which helpers are the preferred owners for common reducer and view-model patterns
- add lint enforcement only after those abstractions are stable enough that bypassing them should no longer be normal

## Constraints

- Do not add lint rules before the shared abstractions are actually good enough to be mandatory.
- Keep domain workflow ownership local; enforce only the reusable UI-model mechanics.
- Treat this as higher-level follow-through on existing TUI boundaries, not as a replacement for the current interaction stack.

## Notes

The scratch note specifically called out modules like `two-pane-state`, `action-target`, and future screen-model abstractions as the kinds of higher-level pathways that may eventually deserve enforcement.

## Related

- [TUI architecture](../../architecture/tui.md)
- [Architectural boundaries](../../architecture/boundaries.md)
