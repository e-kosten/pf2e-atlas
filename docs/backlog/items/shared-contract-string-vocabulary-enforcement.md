# Shared Contract String Vocabulary Enforcement

Status: proposed  
Priority: later  
Owner: unassigned  
Last reviewed: 2026-04-24

## Problem

The repo already centralizes some durable vocabulary through shared domain types and registries, but important contract strings still leak across TUI, search, and lowering code as repeated literals.

This shows up most clearly in places where the same values are used as:

- discriminants for prompt kinds, presentation modes, workflow steps, or action ids
- shared operator names that appear in parsing, rendering, and lowering layers
- user-visible labels that are supposed to stay aligned across multiple surfaces
- cross-module behavior identifiers that should have one clear owning seam

When those strings are duplicated opportunistically, the code drifts even if each individual use is type-correct.

## Desired Outcome

Standardize on central ownership for durable shared contract vocabulary, then enforce those seams once the ownership model is stable.

That follow-through should:

- identify which string families are true shared contracts rather than incidental local copy
- move those vocabularies behind clear owning modules or typed registries
- update downstream code to import shared vocabulary instead of retyping it
- add targeted lint enforcement for those specific families after consolidation

## Constraints

- Do not create a blanket rule against local string literals.
- Do not centralize one-off prose, local-only copy, or incidental error text just for the sake of consistency.
- Prefer ownership by the module that already defines the real behavior contract instead of inventing a generic constants dump.
- Add lint rules only for vocabularies that are stable enough to be mandatory.

## Notes

This is specifically about durable contract strings, not all strings in the repo.

Useful candidate families include:

- prompt and modal presentation discriminants
- shared interaction/action identifiers
- operator vocabularies that span editor, lowering, and search layers
- other cross-module labels or ids whose duplication has already caused drift

## Related

- [Shared UI model boundary enforcement](./shared-ui-model-boundary-enforcement.md)
- [Shared menu/editor behavior contracts](./shared-menu-editor-behavior-contracts.md)
- [Typed seams cleanup](./typed-seams-cleanup.md)
- [TUI architecture](../../architecture/node/tui.md)
- [Architectural boundaries](../../architecture/node/boundaries.md)
