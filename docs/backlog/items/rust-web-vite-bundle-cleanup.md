# Rust Web Vite Bundle Cleanup

Status: proposed
Priority: later
Owner: unassigned
Last reviewed: 2026-06-07

## Problem

The frontend build currently emits a Vite large-chunk warning. This is non-blocking for the prototype, but it should be revisited once the UI shape stabilizes so the web app does not accumulate avoidable startup cost.

The likely contributors include Ant Design, table/form dependencies, icon libraries, and any future record-detail or visualization code.

## Desired Outcome

Evaluate and reduce the Atlas web bundle size where practical.

The pass should consider:

- whether route-level or feature-level dynamic imports are useful;
- whether Ant Design imports and icons are being bundled efficiently;
- whether future heavy features such as graph visualization should load on demand;
- whether Vite/Rolldown configuration should enable code splitting or adjust warning thresholds;
- whether the build output should include a repeatable bundle analysis step.

## Constraints

- Do not optimize prematurely in a way that makes the UI harder to maintain.
- Preserve the selected Ant Design component-library direction unless bundle analysis shows a major blocker.
- Prefer measured bundle analysis over guessing from dependency names.

## Related

- [Architecture overview](../../architecture/overview.md)
- [ADR 0029: Local web app boundary](../../architecture/decisions/0029-local-web-app-boundary.md)
- [Component library decision](../../../web/atlas-ui/docs/component-library-evaluation.md)
