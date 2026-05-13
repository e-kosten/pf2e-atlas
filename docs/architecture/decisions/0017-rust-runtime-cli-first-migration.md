# 0017 Rust Runtime And CLI-First Migration

## Status

Accepted

## Context

PF2e Atlas has been a TypeScript runtime centered on a stdio MCP server, with an Ink terminal workbench and TypeScript-owned index generation. That architecture remains coherent for the current implementation, but the local operating model has shifted toward agent-driven command workflows and dense terminal tooling on a single developer machine.

The Rust migration spikes produced enough evidence to choose a new direction:

- Rust query embeddings match the current MiniLM TypeScript vectors at float precision and have acceptable startup and query latency.
- The artifact-contract spike showed that startup validation should be explicit, versioned, and independent of extension-backed vector tables.
- The canonical-ingest spike found no Rust-specific blocker for deterministic Foundry JSON ingest or SQLite artifact construction; Rust parsed the full vendored corpus quickly, with remaining work centered on parity policy and side-table coverage.
- The CLI-agent spike showed CLI plus skill guidance is a better local direction for compact lookup, safer exact-miss behavior, and direct rule-context answers, while still needing schema/facet discovery and support-record shaping.
- The search-quality spike selected a SQLite-centered hybrid retrieval baseline for the first Rust migration path and deferred Tantivy, LanceDB, heavier rerankers, and model changes.
- The Ratatui spike supports a Rust TUI as the primary terminal direction after the CLI/runtime shell exists.

This changes the target from "Rust runtime over artifacts prepared elsewhere" to "Rust owns the deterministic core path." Python, Node, or TypeScript may still be useful for exploratory analysis, migration comparison, and temporary tooling, but they are not the default long-term owners for canonical ingest, index construction, runtime search, or the primary local agent surface.

## Decision

Adopt a Rust-centered runtime migration with CLI first, Ratatui second, and MCP only as optional compatibility.

The Rust implementation lives under `rust/` as a separate workspace inside this repository. It includes only crates that have active implementation slices:

- `atlas-domain` owns shared Rust contracts and typed runtime vocabulary.
- `atlas-ingest` owns Foundry source loading, normalized ingest records, and SQLite artifact writing.
- `atlas-index` owns artifact/index opening and startup validation.
- `atlas-cli` owns the local `atlas` command surface.

Future crates such as search, embedding, TUI, and MCP should be added only when their first real slice lands. Empty placeholder crates should not be created just to reserve names.

Rust is the target owner for deterministic Foundry ingest, normalized records, source signatures, SQLite table writes, artifact metadata, lookup/search runtime behavior, CLI output contracts, and the Ratatui workbench. Exploratory discovery, clustering, evaluation reports, and parity harnesses may remain in Python, Node, or TypeScript while they are still research or migration aids.

MCP is no longer the architectural center. If retained, `atlas mcp` must be a thin compatibility surface over the Rust runtime and must not introduce MCP-only backend behavior.

## Consequences

The TypeScript runtime remains available during migration, but it is a reference and parity target, not a second long-term implementation owner. New Rust features must not shell out to the TypeScript runtime or wrap the Node MCP server.

The initial Rust workspace must compile, format, lint, and test independently. Its validation gate is:

```bash
cd rust
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
- rule-context and graph traversal
- filter/schema discovery
- SQLite-centered hybrid search
- production Codex skill
- Ratatui workbench
- optional MCP compatibility or retirement
- TypeScript runtime retirement

Cutover is not complete while runtime features have mixed TypeScript and Rust ownership without an explicit parity or retirement plan.
