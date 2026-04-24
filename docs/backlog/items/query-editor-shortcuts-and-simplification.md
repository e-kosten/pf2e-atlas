# Query Editor Shortcuts And Simplification

Status: proposed  
Priority: later  
Owner: unassigned  
Last reviewed: 2026-04-23

## Problem

The canonical query-tree editor should land first in its verbose, explicit form so the core workflow is clear and stable.

That will likely leave follow-up opportunities for simplification once the new editor is in use, such as:

- shortcut actions for common filter patterns
- automatic or semi-automatic regrouping helpers
- simplified rendering for common predicate families
- faster add flows for the most frequent clause kinds

## Desired Outcome

Add deliberate editor shortcuts and simplification affordances only after the verbose/core query editor has landed and its friction points are better understood in practice.

## Constraints

- Do not compromise the canonical query model or reintroduce a second long-lived UI-only state shape.
- Keep any shortcut affordance as sugar that lowers deterministically into the canonical query tree.
- Evaluate simplification based on real editor usage pain points rather than speculative convenience features.

## Related

- [Filter shape convergence](./filter-shape-convergence.md)
- [Remove isUnique metadata](./remove-isunique-metadata.md)
