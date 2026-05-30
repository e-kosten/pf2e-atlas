# 0017 Rust Runtime And CLI-First Migration

## Status

Superseded by ADR 0026 for product surface and workspace layout. Retained as migration sequencing history.

## Context

At the time of this decision, PF2e Atlas was a TypeScript runtime centered on a stdio MCP server, with an Ink terminal workbench and TypeScript-owned index generation. The local operating model had shifted toward agent-driven command workflows and dense terminal tooling on a single developer machine.

The Rust migration spikes produced enough evidence to choose a new direction:

- Rust query embeddings match the current MiniLM TypeScript vectors at float precision and have acceptable startup and query latency.
- The artifact-contract spike showed that startup validation should be explicit, versioned, and independent of extension-backed vector tables.
- The canonical-ingest spike found no Rust-specific blocker for deterministic Foundry JSON ingest or SQLite artifact construction; Rust parsed the full vendored corpus quickly, with remaining work centered on parity policy and side-table coverage.
- The CLI-agent spike showed CLI plus skill guidance is a better local direction for compact lookup, safer exact-miss behavior, and explicit key-based context retrieval, while still needing schema/facet discovery and support-record shaping.
- The search-quality spike selected a SQLite-centered hybrid retrieval baseline for the first Rust migration path and deferred Tantivy, LanceDB, heavier rerankers, and model changes.
- The Ratatui spike supports a Rust TUI as the primary terminal direction after the CLI/runtime shell exists.

This changes the target from "Rust runtime over artifacts prepared elsewhere" to "Rust owns the deterministic core path." Python, Node, or TypeScript may still be useful for exploratory analysis, migration comparison, and temporary tooling, but they are not the default long-term owners for canonical ingest, index construction, runtime search, or the primary local agent surface.

## Decision

Adopt a Rust-centered runtime migration with CLI first and Ratatui second. ADR 0026 later establishes the durable CLI plus first-party skill product surface and the root Rust workspace layout.

The Rust implementation is a workspace inside this repository. It includes only crates that have active implementation slices:

- `atlas-domain` owns shared Rust contracts and typed runtime vocabulary.
- `atlas-record` owns storage-agnostic normalized record DTOs shared by ingest, artifact writing, index loading, and future runtime surfaces. `NormalizedRecord` is the ingest-built construction shape; `PersistedRecord` and `PersistedRecordSet` are durable artifact read shapes.
- `atlas-artifact` owns shared SQLite artifact schema descriptors, ordered record column descriptors, SQL builders for shared table shapes, and artifact metadata contract values.
- `atlas-ingest` owns Foundry source loading, normalized ingest records, build orchestration, and build-input handoff.
- `atlas-index` owns artifact/index opening, startup validation, and SQLite artifact writing.
- `atlas-cli` owns the local `atlas` command surface and agent skill installation.

Future crates such as search, embedding, and TUI should be added only when their first real slice lands. Empty placeholder crates should not be created just to reserve names.

Rust is the target owner for deterministic Foundry ingest, normalized records, source signatures, SQLite table writes, artifact metadata, lookup/search runtime behavior, CLI output contracts, and the Ratatui workbench. Exploratory discovery, clustering, evaluation reports, and parity harnesses may remain in Python, Node, or TypeScript while they are still research or migration aids.

## Consequences

During migration, the TypeScript runtime was a reference and parity target, not a second long-term implementation owner. New Rust features were not expected to shell out to the TypeScript runtime or wrap the Node server.

The initial Rust workspace must compile, format, lint, and test independently. Its validation gate is:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --workspace
```

The initial Rust lint policy should stay close to community defaults: standard Clippy with warnings denied and `unsafe_code` denied in crates. Broader panic-oriented denies such as `unwrap_used` and `expect_used` should be added later only when a crate's runtime API is stable enough that the extra friction is worth it.

The first executable behavior is `atlas index validate`, which proves CLI shape, typed errors, JSON diagnostics, and artifact-contract validation before lookup or search are ported. Index management commands live under the `atlas index` namespace so future product commands such as lookup, search, and browse can occupy the top-level CLI surface.

The migration proceeds by capability gates:

- artifact validation
- exact lookup and record presentation
- graph context retrieval and traversal
- filter/schema discovery
- SQLite-centered hybrid search
- first-party agent skill installation
- Ratatui workbench
- TypeScript runtime retirement

Cutover is not complete while runtime features have mixed TypeScript and Rust ownership without an explicit parity or retirement plan.
