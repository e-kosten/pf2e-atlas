# TypeScript Runtime Architecture

This document describes the TypeScript/Node implementation under `src/`. It remains the established MCP, terminal, search, and editorial runtime while the Rust implementation is built out, but it is no longer the target shape for new Rust runtime decisions. Rust-specific ownership lives in the root [runtime architecture](../runtime.md) and [artifact contract](../artifact-contract.md).

## Status

The TypeScript runtime is the legacy product runtime. It owns the current MCP server, terminal/editorial surfaces, and the mature index-backed service stack. New architecture decisions for Rust should not be inferred from this layout when the Rust docs define a different owner.

## System Shape

```mermaid
flowchart TD
    client["MCP client"] --> stdio["stdio transport"]
    stdio --> server["src/index.ts<br/>MCP server bootstrap"]
    server --> appRuntime["src/app/runtime.ts<br/>runtime assembly"]

    subgraph SharedCore["TypeScript shared runtime and backend core"]
      appRuntime --> dataService["src/data/service.ts<br/>Pf2eDataService"]
      appRuntime --> ranking["src/search/ranking-config.ts<br/>ranking config store"]
      dataService --> catalog["src/data/backend/<br/>catalog + search + rule graph"]
      catalog --> search["src/search/<br/>query analysis + ranking"]
      catalog --> db["Prepared SQLite index"]
      db --> vendor["Vendored PF2E checkout<br/>and prepared embedding assets"]
    end

    server --> toolHandlers["src/server/<br/>tool schemas + presenters"]
    toolHandlers --> dataService

    subgraph TerminalAndEditorial["Terminal and editorial surfaces"]
      tuiRoot["src/tui/app-services.ts<br/>terminal composition"] --> ontology["src/app/ontology-service.ts"]
      tuiRoot --> storage["src/app/storage-service.ts"]
      tuiRoot --> tuiSearch["src/tui/search/service.ts"]
      tuiRoot --> tagUi["src/tags/editorial/ui/<br/>workbench and review controllers"]
      ontology --> dataService
      tuiSearch --> dataService
      storage --> db
      tagUi --> storage
    end

    subgraph Tags["Derived-tag subsystem"]
      authored["src/tags/{ontology,rules,assignments,exemplars}<br/>authored source of truth"]
      reviews["src/tags/reviews/<br/>review queues and reviewed state"]
      runtime["src/tags/runtime/{publication,derivation,matcher,compat}<br/>published runtime ownership"]
      editorial["src/tags/editorial/{state,sessions,writeback,ui}<br/>editorial execution ownership"]
      cli["src/tags/cli/{discovery,evaluation,editorial,shared}<br/>offline entrypoints"]

      authored --> runtime
      reviews --> editorial
      reviews --> runtime
      runtime --> editorial
      cli --> editorial
      cli --> storage
      tagUi --> editorial
    end

    domain["src/domain/<br/>shared contracts and vocabularies"] -. supports .-> dataService
    domain -. supports .-> search
    domain -. supports .-> toolHandlers
    domain -. supports .-> tuiSearch
```

## Ownership Map

- `src/index.ts` is the MCP composition root.
- `src/tui/app-services.ts` is the terminal/editorial composition root.
- `src/app/` wires runtime and app-level facades together.
- `src/data/` owns index-backed catalog, search, and rule-graph access.
- `src/data/indexing/` owns the TypeScript index rebuild pipeline and stage artifacts.
- `src/search/` owns reusable ranked-search mechanics.
- `src/server/` translates MCP tools to backend calls.
- `src/tui/` translates terminal workflows to backend and app services.
- `src/tags/` owns derived-tag runtime, review, editorial, and offline tooling.
- `src/domain/` defines shared TypeScript vocabulary and contracts.

## Boundary Rule

Transport and UI layers stay thin. Shared retrieval behavior flows through `Pf2eDataService` and app-level facades instead of being rebuilt in MCP handlers or TUI screens.

## Relationship To Rust

The Rust implementation does not mirror this structure one-for-one. The Rust crates split source ingest, normalized records, artifact schema, artifact reading, embedding, search orchestration, path/runtime setup, CLI presentation, and sqlite-vec registration into separate crates. When Rust docs and TypeScript docs disagree, treat that as an implementation split, not an inconsistency by itself.
