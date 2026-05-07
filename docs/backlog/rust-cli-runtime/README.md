# Rust CLI Runtime Migration Research

This folder tracks the design-in-progress roadmap and decision spikes for a possible Rust-centered PF2e Atlas runtime.

## Current State

- [Migration roadmap](./migration-roadmap.md)
  Design-in-progress backlog item for a Rust runtime with CLI and TUI as primary local surfaces, optional MCP compatibility, and offline prep tooling kept separate when useful.

## Search And Retrieval Spikes

- [Rust query embedding](./spikes/rust-query-embedding.md)
  Validate Rust runtime query embeddings, model compatibility, latency, startup, and packaging.
- [Search quality bakeoff](./spikes/search-quality-bakeoff.md)
  Build a PF2E-specific eval set and compare retrieval pipelines with better embeddings, lexical alternatives, hybrid ranking, and reranking.
- [Tantivy lexical search](./spikes/tantivy-lexical-search.md)
  Compare Tantivy lexical quality and artifact complexity against SQLite FTS5.
- [LanceDB retrieval store](./spikes/lancedb-retrieval-store.md)
  Evaluate LanceDB as a retrieval artifact for vector, metadata, and hybrid search.

## Runtime Architecture Spikes

- [Rust CLI agent surface](./spikes/rust-cli-agent-surface.md)
  Prototype agent-first `atlas` commands with compact JSON output and a skill-driven workflow.
- [Rust TUI state machine](./spikes/rust-tui-state-machine.md)
  Prototype explicit Ratatui interaction state for list/detail and modal workflows.
- [Artifact contract](./spikes/artifact-contract.md)
  Define the versioned prep/runtime boundary for SQLite, embedding identity, and JSON/JSONL artifacts.

## Follow-Up Decision Spikes

- [Canonical ingest ownership](./spikes/canonical-ingest-ownership.md)
  Decide whether canonical ingest/index build belongs in Rust while Python remains for exploratory analysis.
- [Optional MCP compatibility](./spikes/optional-mcp-compatibility.md)
  Decide whether `atlas mcp` is worth preserving after CLI plus skills are proven.

## Spike Rules

Use small disposable prototypes, but do not mock the risk being evaluated:

- mock product complexity
- use fixture data for speed
- include a representative PF2E slice for relevance decisions
- measure the thing the spike is meant to decide
- finish each spike with a keep, kill, or defer recommendation
