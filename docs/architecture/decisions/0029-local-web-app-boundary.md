# ADR 0029: Local Web App Boundary

## Status

Accepted.

[ADR0064](0064-optional-web-distribution.md) makes web hosting an optional CLI
build feature, enabled by default. The web/app ownership and compatibility
contract below applies to full builds.

The app-layer ownership and pooled web execution decision remain current.
ADR0046 replaces capability-trait retrieval with the concrete search service;
[ADR0049](./0049-search-lifetimes-and-pages.md) supersedes result-window state and
per-call command opens with stateless pages and coherent reader lifetimes.
[ADR0051](./0051-persistent-workspace.md) supersedes retained readers with bounded
workspace sessions. Current runtime and command capabilities are described in
[the runtime architecture](../runtime.md#retrieval-and-application-workflows).

## Context

PF2e Atlas is adding a local interactive web surface on top of the existing Rust search/runtime stack. The web UI needs TypeScript DTOs, HTTP routes, long-lived search/result-window state, and record detail loading, but it should not duplicate CLI internals or leak SQLite/index ownership into the app layer.

The CLI has an explicit no-query-inference capability for short-lived commands,
including stored-vector Similar. Interactive web execution retains pooled,
query-capable execution; it does not automatically fall back to this command mode.

## Decision

Add three app-layer crates:

- `atlas-app-model` for app DTOs, serde contracts, app error codes, readiness views, basic filter state, result-window contracts, record view wrappers.
- `atlas-app-service` for native workflow orchestration over `atlas-runtime` and `atlas-search`; the web service uses it as a long-lived pooled service, while short-lived local CLI clients may choose explicit on-demand modes.
- `atlas-web` for the Axum local HTTP surface, its OpenAPI contract, and embedded frontend assets.

Add `web/atlas-ui` as the TypeScript/React frontend package. It consumes generated app DTOs through a local aggregation file, uses an Orval-generated HTTP client over `/api/*` with an Atlas Fetch transport, and renders the search-to-detail workflow with the selected Ant Design component library.

`atlas web` starts the local service from the CLI. `atlas-web` serves the built
frontend as embedded static assets while reserving `/api/*` for app-service JSON
routes. App-service starts a bounded pool sharing one resolved runtime. Each job
opens a bounded workspace session. Runtime resolution, reference readiness and query
inference retain their own error boundaries; worker startup only creates threads.

Short-lived CLI clients select `OnDemandNoQueryInference` for record/list/filter/graph
reads and stored-vector Similar, or `OnDemandEmbeddings` when text queries need
inference. These are explicit construction capabilities, with no automatic fallback
for `atlas web`. Stored vectors and part inventory remain available without query assets.

`atlas-app-service` must not import or assemble `atlas-index` internals. It adapts app DTOs into `atlas-search` request types and uses narrow retrieval capability traits where practical.

The app-service handle is cloneable for Axum request handlers, but it must not funnel all frontend requests through one unbounded synchronous lane. Shared mutable UI workflow state, such as result-window metadata, stays guarded by explicit app-service synchronization, while retrieval work is submitted to the bounded worker pool and rejected with a retryable busy error when the queue is full.

The annotated HTTP handlers and Rust DTO schemas own the browser contract.
Orval generates Fetch operations and TypeScript models from the exported document.
JSON integers travel as numbers. The transport checks safe integer bounds, and
the frontend API boundary converts fields used as `bigint` by application code.
Orval produces both projections from the same document. Request schemas account
for what Serde accepts, including omissions and explicit nulls. Response schemas
account for what Serde emits. For a DTO used in both directions, declare its
emitted shape on the Rust type and record any accepted omissions or nulls in
`atlas-web/src/schema_directions.rs`. The export uses separate component names
where the shapes differ. See the [frontend contribution workflow](../../../web/atlas-ui/README.md)
for regeneration and freshness commands.

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

`crates/atlas-web/api-contract.json` owns an explicit positive `protocol_revision`.
The annotated Axum router exports `crates/atlas-web/openapi.json` and Orval
generates the browser HTTP client from it. Vite and Rust hash the OpenAPI
document as UTF-8 with CRLF normalized to LF, prefixed with
`atlas-web-openapi`, NUL, the decimal protocol revision and NUL. The identifier
is `atlas-web-v3-<sha256>`. The Rust and TypeScript tests use the same known
vector. OpenAPI description and formatting changes can change the identifier
even when the request and response shapes agree.

Increment the revision when browser-visible behavior changes without a
corresponding OpenAPI change. Examples include changed request interpretation,
defaults, or error behavior. Keep focused Serde and transport regressions
alongside the generated artifact freshness checks.

This identifies a declared wire contract, not semantic equivalence or exact build
identity. Revision discipline and behavioral review are necessary; the guard does
not automatically inspect arbitrary Rust logic for protocol changes. Commit,
platform and executable/UI checksums remain separate artifact provenance. The API
identifier does not depend on the Git commit, checkout path, platform, UI styles,
or build profile.

Vite embeds the expected identifier in the browser bundle and writes
`dist/api-contract.txt`. The Rust build rejects a missing or mismatched bundle
identifier before embedding it. Vite watches only the protocol metadata and
OpenAPI document for compatibility-triggered restarts.
This catches incompatible dist output and a Vite frontend pointed at an
incompatible backend. The separate frontend build receipt establishes freshness;
a compatible frozen backend may still run older implementation code.

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

## Browser error presentation

The thin API client preserves structured app errors and diagnostic messages. Browser
presentation does not render these messages verbatim: the shared feedback owner
selects reviewed task-specific copy from error codes, with safe fallbacks. This
keeps the diagnostic contract useful without exposing SQL, paths, response bodies
or decoding internals. Detailed product rules and examples live in the
[frontend error policy](../../../web/atlas-ui/docs/frontend-guidelines.md#error-feedback-and-recovery).

Transport outages have a distinct browser error type. The root QueryClient reports
read failure only after bounded retries, and reports failed writes without replay.
AppShell owns connection/reference availability and the compatibility notice.
The shared feedback host moves this single notice into the active modal or drawer
so recovery stays reachable inside its focus boundary. Shared Atlas overlay
compositions preserve Ant behavior and release their host when closed. Local
feedback suppresses that classified shared read failure, while independent task
failures and unconfirmed write outcomes remain local and visible. A successful read clears connection status; a ready readiness response clears
reference unavailability. The QueryClient invalidates failed reads only, refreshing
active reads without replaying mutations. App error responses such as a busy
service or unavailable model remain distinct from transport outages.

These presentation decisions do not change Rust error codes, operation ownership,
transaction/receipt semantics, revision guards or the wire compatibility contract.
Browser recovery preserves mounted drafts and distinguishes a failed action,
an unconfirmed write and a successful save whose follow-up read failed.

## Consequences

- Web and future TUI workflows share Rust app contracts and service semantics without forcing the web frontend into WASM.
- TypeScript contracts are generated from Rust DTOs, reducing duplicate frontend interface maintenance.
- The app-service boundary stays native-only and does not need WASM compatibility.
- Web retains pooled query-capable execution. Short-lived CLI clients may explicitly
  prohibit query inference while retaining model-free operations; web never
  automatically adopts that command capability.
- Vite remains the frontend development server and build tool. Normal installed `atlas web` usage does not require Node at runtime because the built frontend is embedded into `atlas-web`; during prototyping, contributors may still run `web/atlas-ui` through Vite and proxy API calls to the local Axum service for hot reload.
- Component-library choice remains a frontend implementation detail above the app-service boundary. The prototype compared Ant Design and Mantine against the same app workflow, then selected Ant Design for the current web UI.
