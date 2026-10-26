# ADR 0029: Local Web App Boundary

## Status

Accepted.

The app-layer ownership and full web readiness decision remain current.
ADR0046 replaces capability-trait retrieval with the concrete search service;
[ADR0049](./0049-search-lifetimes-and-pages.md) supersedes result-window state and
per-call command opens with stateless pages and coherent reader lifetimes.

## Context

PF2e Atlas is adding a local interactive web surface on top of the existing Rust search/runtime stack. The web UI needs TypeScript DTOs, HTTP routes, long-lived search/result-window state, and record detail loading, but it should not duplicate CLI internals or leak SQLite/index ownership into the app layer.

The CLI has no-embeddings and stored-vector retrieval shortcuts for short-lived commands that do not need query embedding generation. Those modes are not appropriate for a long-lived interactive web workflow, where startup should prove the full semantic-search runtime is ready.

## Decision

Add three app-layer crates:

- `atlas-app-model` for app DTOs, serde contracts, app error codes, readiness views, basic filter state, result-window contracts, record view wrappers, and TypeScript generation with `ts-rs`.
- `atlas-app-service` for native workflow orchestration over `atlas-runtime` and `atlas-search`; the web service uses it as a long-lived pooled service, while short-lived local CLI clients may choose explicit on-demand modes.
- `atlas-web` for the Axum local HTTP surface and future static frontend serving.

Add `web/atlas-ui` as the TypeScript/React frontend package. It consumes generated app DTOs through a local aggregation file, uses a thin handwritten API client over `/api/*`, and renders the search-to-detail workflow with the selected Ant Design component library.

`atlas web` starts the local service from the CLI. `atlas-web` serves the built frontend as embedded static assets while reserving `/api/*` for app-service JSON routes. For the web service, app-service starts a bounded pool of retrieval workers; each worker opens a full `AtlasRetrievalService` through `AtlasRuntime::open_retrieval_service`, and web startup fails when artifact/vector/embedding readiness is not satisfied. The web service must not call `open_retrieval_service_no_embeddings`.

App-service may also be started in explicit on-demand retrieval modes for short-lived local CLI clients. The local CLI client uses the no-embeddings app-service mode for record/list/filter/graph reads that do not need query embedding generation, and can use a stored-vector mode for similar-record workflows. These modes are opt-in CLI client construction policy, not fallback behavior for `atlas web`.

`atlas-app-service` must not import or assemble `atlas-index` internals. It adapts app DTOs into `atlas-search` request types and uses narrow retrieval capability traits where practical.

The app-service handle is cloneable for Axum request handlers, but it must not funnel all frontend requests through one unbounded synchronous lane. Shared mutable UI workflow state, such as result-window metadata, stays guarded by explicit app-service synchronization, while retrieval work is submitted to the bounded worker pool and rejected with a retryable busy error when the queue is full.

TypeScript bindings are generated from the app DTO graph and checked into the app-model crate. `cargo test -p atlas-app-model` validates that checked-in bindings are fresh. To deliberately refresh them after DTO changes, run:

```bash
cargo test -p atlas-app-model export_typescript_bindings -- --ignored
```

## Browser/backend compatibility

The web transport owns a compatibility check independent of workspace readiness.
`GET /api/compatibility` responds without opening a database or using an app-service
worker. The browser completes this check before mounting feature components. Missing
support in an older server, or a different contract, blocks feature requests.
An installed frontend automatically reloads the current URL once during startup.
A session-storage marker is written before reloading and cleared only after a
successful handshake. A persistent mismatch or unavailable storage leaves a manual
Reload page action. Development/Vite frontends show a backend/proxy diagnostic and
Retry instead of automatically reloading a pair that cannot repair itself.

`crates/atlas-web/api-contract-inputs.json` selects the app-model source and generated
bindings, app-service source, and HTTP transport source. Vite and the Rust build hash
the same sorted relative paths and UTF-8 contents, normalizing CRLF to LF and framing
each path/content with NUL separators. The identifier is `atlas-web-v1-<sha256>`.
It is conservative source compatibility, not a semantic equivalence proof: even a
test/comment edit under an input directory can require a matching backend. It does
not depend on the Git commit, checkout path, platform, UI styles, or build profile.

Vite embeds the expected identifier in the browser bundle and writes
`dist/api-contract.txt`. The Rust build rejects a missing or mismatched bundle
identifier before embedding it. Vite restarts when a contract input changes.
This catches stale dist output as well as accidentally pointing a current Vite
frontend at an older frozen development backend.

The browser sends `x-atlas-api-contract` on every API request; transport middleware
returns HTTP 409 before executing an advertised incompatible request, including
mutations. Every API response carries the backend identifier and `no-store`.
The browser checks the header before interpreting successful DTOs or a conflict.
Non-browser clients may omit the request header. This is a local compatibility
guard, not authentication or database-generation validation. Existing source and
revision checks still apply independently.

The startup check is cached for this loaded frontend. Ordinary background connection
failures must not unmount the app and discard editor drafts. Later API requests
continue to advertise/check the fingerprint. A detected mismatch becomes sticky in
the shared API client and stops subsequent feature requests. AppShell subscribes to
this failure and offers Reload page while keeping the existing page and editor
drafts mounted. No automatic reload or mutation replay occurs during use. Supported
backends reject incompatible requests before executing mutations; the startup
handshake detects legacy servers without that guard. This cannot make a legacy
server enforce the new request header after an in-flight backend replacement.

HTML remains `no-cache` so page loads validate the entry document. Content-hashed
JavaScript/CSS/font assets retain long-lived immutable caching; recovery does not
clear browser storage or require a cache purge.

## Consequences

- Web and future TUI workflows share Rust app contracts and service semantics without forcing the web frontend into WASM.
- TypeScript contracts are generated from Rust DTOs, reducing duplicate frontend interface maintenance.
- The app-service boundary stays native-only and does not need WASM compatibility.
- Local web startup is stricter than some CLI commands: missing vectors or embedding readiness is a startup error, not a degraded mode. Short-lived CLI app-service clients may opt into explicit on-demand retrieval modes when the command does not need full semantic-search startup.
- Vite remains the frontend development server and build tool. Normal installed `atlas web` usage does not require Node at runtime because the built frontend is embedded into `atlas-web`; during prototyping, contributors may still run `web/atlas-ui` through Vite and proxy API calls to the local Axum service for hot reload.
- Component-library choice remains a frontend implementation detail above the app-service boundary. The prototype compared Ant Design and Mantine against the same app workflow, then selected Ant Design for the current web UI.
