# Rust Query Embedding Spike

Status: proposed  
Priority: soon  
Owner: unassigned  
Last reviewed: 2026-05-06

## Question

Can the Rust runtime compute query embeddings locally with acceptable quality, startup time, latency, packaging cost, and compatibility with offline document embeddings?

## Motivation

Semantic search is a core feasibility gate for a Rust runtime. If Rust cannot produce query vectors in the same embedding space as the prepared index, the runtime either loses semantic search or reintroduces a Python/Node runtime sidecar.

## Prototype Scope

- Build a disposable Rust prototype under `scratch/spikes/` or an experimental worktree.
- Test the then-current MiniLM parity contract: `Xenova/all-MiniLM-L12-v2`, mean pooling, normalized vectors, 384 dimensions.
- Test at least one likely upgrade candidate such as `BAAI/bge-small-en-v1.5`.
- Use a tiny embedding fixture for fast checks and a representative PF2E query slice for relevance checks.
- Compare against the current TypeScript embedding provider output where feasible.

## Do Not Mock

- model loading
- tokenizer behavior
- pooling
- normalization
- query/document prefixes
- vector dimensions
- query latency
- startup latency
- packaging and model-cache behavior

## Outputs

- measured startup and per-query latency
- model/cache size notes
- compatibility notes against current TypeScript vectors
- candidate model recommendation
- recommendation for Rust runtime embedding provider: keep, kill, or defer

## Migration Dependency

The migration roadmap should not choose a Rust search runtime until this spike confirms a viable runtime embedding path or explicitly accepts semantic-search degradation.
