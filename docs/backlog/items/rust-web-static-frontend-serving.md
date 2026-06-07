# Rust Web Static Frontend Serving

Status: deferred
Priority: later
Owner: unassigned
Last reviewed: 2026-06-07

## Problem

During prototyping, `web/atlas-ui` runs through Vite and proxies `/api/*` to the local Axum service. That is appropriate for development, but it is not the long-term user-facing shape for `atlas web`.

Eventually users should be able to launch the local web UI from `atlas web` without manually starting a separate frontend dev server.

## Desired Outcome

Serve the built frontend through `atlas-web` for normal user workflows, while preserving Vite as the development path.

The design should decide:

- how the frontend build artifact is produced and packaged;
- whether `atlas web --open` opens the served app route rather than only the API readiness page;
- how cache headers, asset paths, and SPA fallback routing should work;
- how contributor workflows distinguish dev-server mode from built-static mode;
- whether frontend build validation belongs in repo verification or a narrower web gate.

## Constraints

- Keep `atlas-web` as transport/static-serving glue; it should not own frontend state or retrieval semantics.
- Do not block current Vite proxy development while static serving is deferred.
- Avoid embedding stale generated frontend artifacts without a clear refresh workflow.

## Related

- [Architecture overview](../../architecture/overview.md)
- [ADR 0029: Local web app boundary](../../architecture/decisions/0029-local-web-app-boundary.md)
