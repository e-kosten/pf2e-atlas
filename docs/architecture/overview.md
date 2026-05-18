# Architecture Overview

This is the front door for the project architecture. Read it first when you need to understand which implementation owns the area you are changing, then follow the implementation-specific and subsystem-specific documents.

## What This System Is

This repository currently has two implementation shapes:

- the established TypeScript/Node runtime under `src/`
- the target Rust runtime under `rust/`

The TypeScript runtime still owns the current mature MCP, terminal, and editorial product surfaces. The Rust runtime owns the new deterministic ingest/artifact/search direction and is where new Rust-specific architecture decisions should land.

Do not silently apply the TypeScript layout to Rust. The TypeScript implementation is service/folder-oriented around `Pf2eDataService`; the Rust implementation is crate-oriented around explicit ownership of records, artifacts, ingest, index reading, embedding, search, runtime setup, and CLI presentation.

## At A Glance

Use these docs by implementation:

- [TypeScript runtime architecture](./node/runtime.md): current `src/` MCP/TUI/editorial runtime shape.
- [Runtime architecture](./runtime.md): current `rust/` crate ownership, ingest flow, content projections, and runtime query flow.
- [Artifact contract](./artifact-contract.md): Rust SQLite schema, table families, validation contract, and embedding/vector artifact boundary.
- [Architectural boundaries](./node/boundaries.md): enforced TypeScript layering and repo-wide editing rules.
- [Search architecture](./node/search.md), [TUI architecture](./node/tui.md), and [Editorial architecture](./node/editorial.md): mature TypeScript subsystem docs.
- [Architecture decisions](./decisions/README.md): accepted decisions, including Rust ADRs 0017-0020.

The shortest useful TypeScript mental model is:

- `src/index.ts` is the MCP composition root
- `src/tui/app-services.ts` is the terminal/editorial composition root
- `src/app/` wires runtime and app-level facades together
- `src/app/search-discovery-service.ts` is the public app facade for shared search discovery; focused discovery owners live under `src/app/search-discovery/`
- `src/data/` owns index-backed catalog, search, and rule-graph access
- `src/data/indexing/` owns the typed index rebuild pipeline and stage artifacts
- `src/domain/search-request-types.ts` owns `SearchRequest`, the shared semantic search contract
- `src/domain/metadata-field-catalog.ts`, `src/domain/metadata-field-types.ts`, `src/domain/search-filter-metadata.ts`, and `src/domain/search-request-types.ts` own the shared metadata filter vocabulary carried by `SearchRequest`
- `src/search/` owns reusable ranked-search mechanics
- `src/server/` translates MCP tools to backend calls
- `src/tui/` translates user interaction flows to backend and app services
- `src/tags/runtime/` owns published derived-tag runtime assembly
- `src/tags/reviews/` owns durable review registries and reviewed discovery state
- `src/tags/editorial/` owns editorial state, session, writeback, and UI workflows
- `src/tags/cli/` groups offline discovery, evaluation, and editorial entrypoints
- `docs/architecture/artifact-contract.md` owns the first Rust SQLite artifact contract and validation diagnostic families
- `src/search/filters/` owns execution-time metadata normalization, validation, and record-level matching; public metadata field semantics stay in `src/domain/metadata-field-catalog.ts`
- `src/data/backend/search-sql.ts` and `src/data/backend/metadata-search-sql.ts` own SQL-facing filter assembly and metadata predicate SQL construction
- `src/data/metadata-row-projection.ts` owns metadata row selection and hydration mapping for normalized records
- `src/search/request-compilation.ts` owns the lowering from `SearchRequest` into search-execution filters
- `src/tags/runtime.ts`, `src/tags/editorial.ts`, and `src/tags/editorial-ui.ts` are the approved non-tag tag facades
- `src/domain/` defines shared vocabulary and contracts
- `src/shared/` stays intentionally small and only holds true cross-layer primitives

The shortest useful Rust mental model is:

- `atlas-cli` owns command parsing, output, progress, exit codes, and agent skill installation
- `atlas-runtime` owns path/setup policy
- `atlas-search` owns runtime search orchestration
- `atlas-index` owns artifact validation, row readers, filter compilation, and vector SQL
- `atlas-embedding` owns model catalog, embedding text rendering, token budgeting, document units, and query/document vectors
- `atlas-ingest` owns source loading, Foundry parsing, normalization, enrichment, generation, reference resolution, retrieval visibility, and artifact writing
- `atlas-record` owns normalized records, `ContentDocument`, presentation contracts, FTS projection, and section-tree projection
- `atlas-artifact` owns physical SQLite table/column descriptors and contract constants
- `atlas-domain` owns shared Rust request/filter/output vocabulary
- `atlas-sqlite-vec` owns sqlite-vec registration and capability probing

If you remember only one rule, remember this one: first identify which implementation you are changing. In TypeScript, transport and UI layers should stay thin and shared retrieval behavior should flow through `Pf2eDataService` and app-level facades. In Rust, CLI and future surfaces should stay thin and durable behavior should flow through the crate that owns the concern.

## System Overview

```mermaid
flowchart TD
    pf2e["Vendored PF2E source"] --> nodeIndex["TypeScript index builder<br/>src/data/indexing"]
    pf2e --> rustIngest["Rust ingest<br/>atlas-ingest"]

    nodeIndex --> legacyDb["Prepared SQLite index<br/>TypeScript contract"]
    rustIngest --> rustDb["Rust SQLite artifact<br/>pf2e-atlas-artifact/v1"]

    subgraph NodeRuntime["TypeScript/Node runtime under src/"]
      mcp["MCP server<br/>src/index.ts + src/server"]
      tui["Terminal/editorial UI<br/>src/tui + src/tags"]
      data["Pf2eDataService<br/>src/data"]
      tsSearch["Search mechanics<br/>src/search"]
      mcp --> data
      tui --> data
      data --> tsSearch
      data --> legacyDb
    end

    subgraph RustRuntime["Rust runtime under rust/"]
      cli["atlas-cli"]
      runtime["atlas-runtime"]
      rustSearch["atlas-search"]
      index["atlas-index"]
      embedding["atlas-embedding"]
      record["atlas-record"]
      artifact["atlas-artifact"]
      cli --> runtime
      cli --> rustSearch
      rustSearch --> index
      rustSearch --> embedding
      index --> rustDb
      embedding --> rustDb
      rustIngest --> record
      rustIngest --> artifact
      index --> record
      index --> artifact
    end

    rustFuture["Future Rust TUI/MCP surfaces"] -. should compose through .-> rustSearch
```

## Execution Surfaces

For the TypeScript implementation, see [TypeScript runtime architecture](./node/runtime.md) for the full runtime diagram and the detailed `src/` ownership map.

For the Rust implementation, see [runtime architecture](./runtime.md) for crate diagrams, ingest flow, content/search/reference projections, and runtime query flow.

### MCP Server

The current public product surface is the TypeScript stdio MCP server in `src/index.ts`. It:

1. loads config and ranking state via `src/app/runtime.ts`
2. creates one long-lived `Pf2eDataService`
3. registers lookup, search, and rules tools from `src/server/`
4. serves requests over stdio using thin tool handlers

The server layer should own wire concerns such as schemas, descriptions, and response presentation. It should not own ranking policy, SQL access, or new storage lifecycles.

A future Rust MCP surface should be thin over the Rust runtime crates. It should not copy TypeScript service internals into Rust or open SQLite/model resources directly from transport code.

### Terminal UI

The terminal application composes on top of the same runtime through `src/tui/app-services.ts`. It adds:

- ontology browsing via `src/app/ontology-service.ts`
- search workflow orchestration via `src/tui/search/service.ts` and `src/tui/search/`
- terminal framework and navigation state in `src/tui/framework/` and nearby screens

The TUI is a consumer of the shared runtime, not a second backend.

A future Rust TUI should consume the Rust runtime crates through search/index/runtime/presentation boundaries rather than rebuilding artifact or embedding access in screen code.

### Editorial And Tagging Tooling

The editorial subsystem under `src/tags/` is large because it supports assignment logic, candidate discovery, migration sessions, review queues, and CLI workflows. Architecturally, what matters here is:

- authored truth lives in `ontology/`, `rules/`, `assignments/`, `exemplars/`, and `reviews/`
- published runtime ownership is split across `runtime/publication/`, `runtime/derivation/`, `runtime/matcher/`, and `runtime/compat/`
- durable reviewed discovery negatives now live under `src/tags/reviews/discovery-reviewed-records.ts`, alongside the other review registries
- editorial execution is split by concern under `editorial/state/`, `editorial/sessions/`, `editorial/writeback/`, and `editorial/ui/`
- offline tooling is grouped under `cli/discovery/`, `cli/evaluation/`, `cli/editorial/`, and `cli/shared/`
- non-editorial code should prefer `src/tags/runtime.ts`, `src/tags/editorial.ts`, or `src/tags/editorial-ui.ts` over arbitrary imports into tag leaf modules

See [`editorial.md`](./node/editorial.md) for the deeper breakdown of the editorial subsystem.

## TypeScript Layer Responsibilities

The sections below describe the established TypeScript implementation under `src/`. For Rust crate responsibilities, use [runtime architecture](./runtime.md).

### `src/domain/`

Owns low-level shared vocabulary and contracts:

- categories and subcategories
- metadata semantics and predicate specs
- search and rule graph types
- ontology contracts and related shared type definitions

There is no approved broad `src/domain/index.ts` import path. Import the owning `src/domain/*` module directly when code needs a domain contract.

This layer should stay free of transport, UI, and storage-lifecycle behavior.

### `src/shared/`

Owns a tiny set of true cross-layer primitives.

- keep stable low-level helpers here only when multiple layers genuinely share them
- do not treat `src/shared/` as the default home for convenience helpers once `app`, `data`, or `search` clearly owns the concern
- moved non-tag helpers should stay with their owner modules instead of growing compatibility imports back into shared barrels

### `src/data/`

Owns index-backed retrieval and normalized record access:

- `Pf2eDataService` as the main backend facade
- backend catalog, search, rule-graph, and generic reference-edge services in `src/data/backend/`
- normalization, row decoding, indexing, and schema helpers
- typed index rebuild stages under `src/data/indexing/`; `build-index.ts` owns the transaction and stage order, while focused stage modules own source loading, normalization, family assignment, reference resolution, canonicalization, record/search-text writing, embedding writing, and catalog writing

This layer also owns its record/raw-value helper pathways such as normalization, nested-value extraction, and formatting helpers that were previously reachable through broader shared modules.

This is the core boundary between the rest of the application and low-level SQLite-backed access.

### `src/search/`

Owns reusable ranked-search mechanics:

- query analysis and normalization
- ranking configuration
- runtime search execution
- lexical and semantic scoring coordination

Search-specific primitives such as limit/offset clamping and lexical scoring belong here once they are part of the search runtime rather than general shared infrastructure.

If logic is reusable across MCP and TUI search paths, it probably belongs here or in `src/data/backend/`, not in a handler or screen.

### `src/app/`

Owns application-level composition and facades:

- runtime assembly in `runtime.ts`
- storage boundary in `storage-service.ts`
- ontology assembly in `ontology-service.ts` and `ontology/`
- search-semantics ontology assembly in `ontology/search-semantics/`, where domain, field, metric, pack, value-node, child-source, query, and label builders stay split by ownership
- shared entity-page document and page-text presentation ownership in `ontology/entity-page.ts`, with normalized page facts projected by `ontology/entity-page-facts.ts`, `ontology/entity-page-service.ts` as the relation-aware app facade, and `ontology/presenter.ts` as the durable plain-line presentation seam for non-page consumers
- page-relation grouping and seeded drill preparation in `page-relations-service.ts`

This layer exists so product surfaces do not have to know how to construct storage, vocabulary, or shared models themselves.

### `src/server/`

Owns MCP transport concerns:

- tool registration
- schema descriptions
- response shaping and presentation

It should translate between the MCP SDK and `Pf2eDataService`, not create competing backend logic.

### `src/tui/`

Owns terminal interaction concerns:

- user workflows and state machines
- terminal framework helpers
- UI-facing service adapters

It should consume explicit facades rather than reach directly into storage or low-level backend internals.

### `src/tags/`

Owns the editorial subsystem:

- authored tag knowledge in `ontology/`, `rules/`, `assignments/`, `exemplars/`, and `reviews/`
- published runtime assembly in `runtime/publication/`, `runtime/derivation/`, `runtime/matcher/`, and `runtime/compat/`
- editorial state, session, writeback, and review UI flows under `editorial/`
- discovery and evaluation tooling plus grouped CLI entrypoints

This area evolves faster than the public MCP surface, so its internal structure may change more often. The important boundary is stable entrypoints and respect for shared storage/runtime seams.

## Runtime Composition

There are two composition roots and one shared backend runtime.

### Shared Backend Runtime

`src/app/runtime.ts` is the common assembly layer. It:

1. loads configuration
2. opens the ranking config store
3. calls `Pf2eDataService.load(...)`
4. returns the shared runtime handle used by higher-level surfaces

That runtime bundles config, startup warnings, pack and record stats, and a close hook. Both the server and the terminal stack build on top of it.

### MCP Composition Root

`src/index.ts` is intentionally small. Its job is to:

1. load the shared runtime
2. create the `McpServer`
3. register the tool families in `src/server/`
4. connect the stdio transport

If `src/index.ts` starts to contain feature logic, that is usually a sign the logic belongs elsewhere.

### Terminal Composition Root

`src/tui/app-services.ts` takes the shared runtime and layers on:

- `createPf2eApplicationStorageService` for explicit direct-index workflows
- `createPf2eApplicationOntologyService` for the cached search-semantics browse model
- `createPf2eTerminalSearchService`
- tag workbench services wired through storage-backed helpers

This is what lets the terminal/editorial surface reuse the backend while still having terminal-specific service contracts.

## MCP Request Flow

The request lifecycle below is the main product path to keep in mind when changing lookup, search, or rules behavior.

```mermaid
sequenceDiagram
    participant Client as MCP Client
    participant Transport as stdio Transport
    participant Server as McpServer
    participant Handler as src/server/* tool handler
    participant Service as Pf2eDataService
    participant Backend as Backend search/catalog/rule graph
    participant Store as SQLite index

    Client->>Transport: tool call
    Transport->>Server: parsed MCP request
    Server->>Handler: invoke registered tool
    Handler->>Service: call lookup/search/rule API
    Service->>Backend: delegate by capability
    Backend->>Store: execute indexed reads
    Store-->>Backend: rows and linked data
    Backend-->>Service: normalized result models
    Service-->>Handler: typed response payload
    Handler-->>Server: presented MCP content + structured content
    Server-->>Transport: MCP response
    Transport-->>Client: tool result
```

A few implications follow from that flow:

- request schemas belong in `src/server/`
- reusable retrieval behavior belongs below the tool handlers
- normalization should happen once in shared backend paths, not once per caller
- result presentation can differ by surface, but the underlying data access path should stay centralized

## Shared Data And Storage Model

The project is intentionally offline-first at runtime:

- PF2E source data is expected locally under `vendor/pf2e` by default
- embeddings are prepared locally when semantic search is enabled
- the server and TUI read from a prepared SQLite index rather than rebuilding it on startup

That means normal request handling is read-only and low-churn:

- startup validates and opens prepared resources
- steady-state request handling executes indexed reads
- refresh and rebuild operations happen explicitly through maintenance commands, not during normal MCP traffic

The most important storage split is:

- long-lived data runtime lives behind `Pf2eDataService`
- short-lived direct index access for app workflows goes through `src/app/storage-service.ts`

This keeps `DatabaseSync` construction from spreading through the codebase.

## Architectural Intent

Recent refactors point in a clear direction that future edits should preserve:

- prefer focused modules over large multipurpose service files
- prefer facades between layers over direct leaf imports
- prefer readonly shared models where consumers should browse rather than mutate
- prefer lint-enforced boundaries once a shared pathway is mature

The details may move, but those design choices are the stable intent.

## How To Navigate The Architecture Docs

Use this page as the map, then jump to the document closest to your change.

```mermaid
flowchart TD
    overview["overview.md<br/>start here"] --> boundaries["boundaries.md<br/>cross-cutting rules"]
    overview --> search["search.md<br/>retrieval and ranking"]
    overview --> tui["tui.md<br/>terminal composition"]
    overview --> editorial["editorial.md<br/>tagging and review workflows"]
    overview --> extending["extending.md<br/>where new behavior belongs"]
    overview --> decisions["decisions/README.md<br/>ADR index and design notes"]
```

### Architecture Reading Order

If you are new to the repo, use this sequence:

1. read [`README.md`](../../README.md)
2. read this overview
3. read [`boundaries.md`](./node/boundaries.md)
4. jump to the focused doc for your subsystem:
   - [`search.md`](./node/search.md)
   - [`tui.md`](./node/tui.md)
   - [`editorial.md`](./node/editorial.md)
   - [`extending.md`](./node/extending.md)
   - [`decisions/README.md`](./decisions/README.md)
5. inspect the relevant composition root:
   - `src/index.ts` for MCP changes
   - `src/tui/app-services.ts` for terminal/editorial changes
6. find the owning facade before touching leaf modules

### Which Document To Open Next

- Open [`boundaries.md`](./node/boundaries.md) when you need to know what is intentionally restricted or lint-enforced.
- Open [`search.md`](./node/search.md) when you are changing ranking, filter behavior, or retrieval flow.
- Open [`tui.md`](./node/tui.md) when you are changing terminal composition, navigation, or TUI service seams.
- Open [`editorial.md`](./node/editorial.md) when you are changing derived-tag workflows, review tooling, or migration flows.
- Open [`extending.md`](./node/extending.md) when you are deciding where a new feature or abstraction belongs.
- Open [`decisions/README.md`](./decisions/README.md) when a design choice depends on previous architectural commitments.

## Editing Guidance

When adding or moving behavior, prefer the lowest layer that can own the work without importing higher-level concerns.

- MCP-only presentation changes belong in `src/server/`
- reusable retrieval logic belongs in `src/search/` or `src/data/backend/`
- app-level service wiring belongs in `src/app/`
- terminal interaction behavior belongs in `src/tui/`
- stable shared vocabulary belongs in `src/domain/`

If a new shared abstraction is meant to become the normal path through the codebase, update the lint rules once that path is stable enough to enforce.

## Validation Expectations

The full repo gate is:

```bash
cd scripts && npm run verify
```

For documentation-only changes, still run at least the project build and test suite before landing work so the branch state is known-good.
