# ADR 0029: Local Web App Boundary

## Status

Accepted.

## Context

PF2e Atlas is adding a local interactive web surface on top of the existing Rust search/runtime stack. The web UI needs TypeScript DTOs, HTTP routes, long-lived search/result-window state, and record detail loading, but it should not duplicate CLI internals or leak SQLite/index ownership into the app layer.

The CLI has a no-embeddings retrieval shortcut for short-lived commands that do not need semantic search. That shortcut is not appropriate for a long-lived interactive web workflow, where startup should prove the full semantic-search runtime is ready.

## Decision

Add three app-layer crates:

- `atlas-app-model` for app DTOs, serde contracts, app error codes, readiness views, basic filter state, result-window contracts, record view wrappers, and TypeScript generation with `ts-rs`.
- `atlas-app-service` for long-lived native workflow orchestration over `atlas-runtime` and `atlas-search`.
- `atlas-web` for the Axum local HTTP surface and future static frontend serving.

Add `web/atlas-ui` as the TypeScript/React frontend package. It consumes generated app DTOs through a local aggregation file, uses a thin handwritten API client over `/api/*`, and contains the first Ant Design vs Mantine prototype screens for the search-to-detail workflow.

`atlas web` starts the local service from the CLI. The app service opens one full `AtlasRetrievalService` through `AtlasRuntime::open_retrieval_service` and fails startup when artifact/vector/embedding readiness is not satisfied. It must not call `open_retrieval_service_no_embeddings`.

`atlas-app-service` must not import or assemble `atlas-index` internals. It adapts app DTOs into `atlas-search` request types and uses narrow retrieval capability traits where practical.

The initial implementation may use a cloneable app-service handle backed by a dedicated worker thread that owns retrieval state and result-window handles. Axum request handlers call through that handle instead of reopening resources per request.

TypeScript bindings are generated from the app DTO graph and checked into the app-model crate. `cargo test -p atlas-app-model` validates that checked-in bindings are fresh. To deliberately refresh them after DTO changes, run:

```bash
cargo test -p atlas-app-model export_typescript_bindings -- --ignored
```

## Consequences

- Web and future TUI workflows share Rust app contracts and service semantics without forcing the web frontend into WASM.
- TypeScript contracts are generated from Rust DTOs, reducing duplicate frontend interface maintenance.
- The app-service boundary stays native-only and does not need WASM compatibility.
- Local web startup is stricter than some CLI commands: missing vectors or embedding readiness is a startup error, not a degraded mode.
- Static frontend serving remains future work. During prototyping, `web/atlas-ui` runs through Vite and proxies API calls to the local Axum service.
- Component-library choice remains a frontend implementation detail above the app-service boundary; the current prototype compares Ant Design and Mantine against the same app workflow.
