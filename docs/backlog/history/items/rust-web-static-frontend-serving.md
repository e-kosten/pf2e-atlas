# Rust Web Static Frontend Serving

Status: done  
Priority: later  
Owner: unassigned  
Last reviewed: 2026-06-16

## Problem

This item is complete. Normal `atlas web` usage now serves the built React frontend through the Rust `atlas-web` crate instead of requiring users to start the Vite development server.

## Desired Outcome

The landed outcome is:

- `web/atlas-ui` still uses Vite for frontend development and build output.
- `atlas-web` embeds the built `web/atlas-ui/dist` assets with `rust-embed`.
- `atlas web` serves `/` and frontend routes from the embedded app while preserving `/api/*` for Axum app-service routes.
- hashed frontend assets under `assets/` receive long-lived immutable cache headers, while `index.html` remains no-cache.
- local just targets, CI, and release workflows build or verify the frontend before compiling or packaging the Rust binary.

## Constraints

- Node, npm, and Vite are build-time/development-time tools only; the installed runtime does not require them.
- `atlas-web` remains transport and static-serving glue. Frontend state stays in `web/atlas-ui`; retrieval and workflow semantics stay in `atlas-app-service`.
- The Vite dev-server path remains available for hot-reload frontend work.

## Related

- [Architecture overview](../../../architecture/overview.md)
- [ADR 0029: Local web app boundary](../../../architecture/decisions/0029-local-web-app-boundary.md)
