# ADR 0009: Shared List/Detail Presentation Layer

- Status: Accepted
- Date: 2026-04-20

## Context

ADR 0006 established shared TUI interaction contracts, but several non-trivial screens were still assembling the same list/detail presentation mechanics independently:

- pane sizing and detail-window measurement
- list/detail screen-model assembly
- footer/status composition around shared action tables
- shared interaction-context router setup for list, detail, optional text-entry, and optional action-target surfaces

Search, filter explorer, and derived-tag review all needed those mechanics, but they still owned parallel presentation glue above the lower-level router and rendering primitives.

That duplication made the TUI harder to evolve consistently and raised the cost of adding new list/detail screens without actually improving feature ownership.

## Decision

Introduce one shared TUI presentation layer for list/detail screens in `src/tui/list-detail-presentation.ts`.

That layer owns:

- list/detail measurement helpers for body height, detail width, detail scroll bounds, and visible detail lines
- shared screen-model assembly into `TerminalPaneScreen` / `TerminalTwoPaneScreen` props
- shared interaction-context setup for list, detail, optional text-entry, and optional action-target routing

That layer does not own:

- feature-domain actions or their semantics
- feature reducers beyond the shared presentation state they already compose
- command palette contents
- detail/body rendering content
- async workflows such as executing searches, opening routes, or importing review sessions

The intended usage model is:

- shared layer handles repeated presentation mechanics
- feature controllers supply action tables, list/detail content builders, and intent mapping into local workflow or reducer actions

## Consequences

- Search result-reader, filter explorer, and review screens share one list/detail presentation seam instead of maintaining parallel metric and router glue.
- Feature-specific domain workflows remain local, so the shared layer does not flatten unlike surfaces into one generic reducer.
- New list/detail screens should start from the shared presentation layer when they need the same pane sizing, footer/help, and routing mechanics.
- For list/detail screens that fit the shared presentation contract, boundary docs and lint rules now treat that layer as the mandatory route instead of leaving it convention-only.
