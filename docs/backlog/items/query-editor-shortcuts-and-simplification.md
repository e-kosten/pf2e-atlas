# Query Editor Shortcuts And Simplification

Status: proposed  
Priority: later  
Owner: unassigned  
Last reviewed: 2026-04-23

## Problem

The canonical query-tree editor is now landed in its verbose, explicit form, which makes the core workflow clear but also exposes the places where follow-up simplification may be worthwhile.

That already leaves follow-up opportunities such as:

- shortcut actions for common filter patterns
- automatic or semi-automatic regrouping helpers
- simplified rendering for common predicate families
- faster add flows for the most frequent clause kinds

## Desired Outcome

Add deliberate editor shortcuts and simplification affordances now that the verbose/core query editor is in use and its friction points can be judged from real usage rather than speculation.

## Constraints

- Do not compromise the canonical query model or reintroduce a second long-lived UI-only state shape.
- Keep any shortcut affordance as sugar that lowers deterministically into the canonical query tree.
- Evaluate simplification based on real editor usage pain points rather than speculative convenience features.

## Related

- [Filter shape convergence](../history/items/filter-shape-convergence.md)
- [Remove isUnique metadata](../history/items/remove-isunique-metadata.md)
