# Structured Editor Metric Availability Ownership

Status: proposed  
Priority: later  
Owner: unassigned  
Last reviewed: 2026-05-05

## Problem

The structured query editor preflights actor and item metric availability before opening metric-family prompts. That currently behaves like prompt-local option filtering rather than route classification, but it sits close enough to route and loading ownership that future changes could mistake it for an acceptable ad hoc routing path.

## Desired Outcome

Clarify the durable owner for metric-family availability in structured-editor metric and metric-comparison flows:

- decide whether metric-family availability belongs in prompt-local filtering, the route catalog, or the shared explorer/metric selection owner
- if it remains prompt-local, document and test that it is availability filtering only, not route classification
- if it moves, pass metric-family availability through the shared route/explorer owner rather than loading it directly from prompt actions
- add lint enforcement only if a concrete bypass pattern emerges

## Constraints

- Keep metric and metric-comparison routes as leaf routes.
- Do not reintroduce exact-node fallback routing or a second route path for metric clauses.
- Preserve the shared explorer child surface for metric-key selection.

## Related

- [TUI architecture](../../architecture/node/tui.md)
- [Architecture boundaries](../../architecture/node/boundaries.md)
