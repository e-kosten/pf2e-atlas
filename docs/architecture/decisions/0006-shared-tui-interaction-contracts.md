# ADR 0006: Shared TUI Interaction Contracts

- Status: Accepted
- Date: 2026-04-20

## Context

The TUI had multiple screens that were solving the same interaction problems independently:

- raw event decoding and list/detail navigation were still being handled in feature-local code
- footer and help text could drift away from the actual bindings in force
- action-target pages and command-oriented pages did not consistently route through the same shared interaction vocabulary
- editorial review actions still had bespoke routing even after the shared interaction stack had been introduced elsewhere

That made the TUI harder to reason about and directly conflicted with the architecture goal that shared screen behaviors should be selected and composed, not reimplemented per screen.

## Decision

Treat TUI interaction routing, action-target behavior, and help/footer derivation as shared contracts that feature screens consume instead of locally redefining.

In practice this means:

- feature screens route input through shared interaction helpers such as `src/tui/interaction-context-router.ts`
- menu-style and action-target surfaces derive footer/help output from the same interaction action tables they execute
- action-target pages use the shared action-target model and focus semantics instead of bespoke direct-action handling
- editorial and search surfaces should adopt the same shared interaction routing path once their workflows are migrated

## Consequences

- Vim keys, arrow keys, action-target entry/exit, and common screen actions stay consistent across migrated surfaces.
- Footer and help copy are less likely to drift because they are generated from the same binding model that routes input.
- New TUI flows should extend shared interaction helpers or screen primitives when they need a new common behavior, rather than branching on raw terminal events in feature code.
- Lint rules remain an expected part of closing these migrations once the shared path is stable enough to be mandatory.
