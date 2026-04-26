# ADR 0009: Shared List/Detail Presentation Layer And Behavior Contracts

- Status: Accepted
- Date: 2026-04-20

## Context

ADR 0006 established shared TUI interaction contracts, but several non-trivial screens were still assembling the same list/detail presentation mechanics independently:

- pane sizing and detail-window measurement
- list/detail screen-model assembly
- footer/status composition around shared action tables
- shared interaction-context router setup for list, detail, optional text-entry, and optional action-target surfaces
- rightward dead-end policy for list/detail screens that already fit the shared presentation seam

Search, filter explorer, and derived-tag review all needed those mechanics, but they still owned parallel presentation glue above the lower-level router and rendering primitives.

That duplication made the TUI harder to evolve consistently and raised the cost of adding new list/detail screens without actually improving feature ownership. It also left behavior-level drift around what rightward confirm means, when a dead end should notify, and whether rightward failure may change focus.

## Decision

Introduce one shared TUI list/detail layer with presentation owners in `src/tui/list-detail-presentation.ts` and shared behavior contracts in `src/tui/list-detail-behavior.ts`.

That layer owns:

- list/detail measurement helpers for body height, detail width, detail scroll bounds, and visible detail lines
- shared screen-model assembly into `TerminalPaneScreen` / `TerminalTwoPaneScreen` props
- shared interaction-context setup for list, detail, optional text-entry, and optional action-target routing
- shared rightward list behavior contracts for `drill`, `open`, `preview`, and `none`
- shared dead-end routing through the footer-banner notification seam, including preview-already-visible behavior
- explicit-only pane-focus policy for screens that fit that behavior contract

That layer does not own:

- feature-domain actions or their semantics
- feature reducers beyond the shared presentation state they already compose
- action rail contents
- detail/body rendering content
- async workflows such as executing searches, opening routes, or importing review sessions

The intended usage model is:

- shared layer handles repeated presentation mechanics
- feature controllers supply action tables, list/detail content builders, and successful intent mapping into local workflow or reducer actions
- qualifying callers map row state into shared behavior concepts like right intent, destination availability, and dead-end policy instead of hand-rolling those decisions in screen code

## Consequences

- Search result-reader, filter explorer, and review screens share one list/detail presentation seam instead of maintaining parallel metric and router glue.
- The search result-reader and filter explorer inspect/compose flows also share one rightward behavior contract instead of maintaining parallel dead-end and focus-fallback policy.
- Feature-specific domain workflows remain local, so the shared layer does not flatten unlike surfaces into one generic reducer.
- Pane-focus changes stay explicit-only for screens that fit the behavior contract, so rightward dead ends no longer act like implicit focus changes.
- Derived-tag review keeps the shared presentation seam but stays outside the rightward behavior contract because its main rightward interaction is an action-target workflow rather than list-row confirm behavior.
- New list/detail screens should start from the shared presentation layer when they need the same pane sizing, footer/help, and routing mechanics.
- For list/detail screens that fit the shared presentation contract, boundary docs and lint rules now treat that layer as the mandatory route instead of leaving it convention-only.
