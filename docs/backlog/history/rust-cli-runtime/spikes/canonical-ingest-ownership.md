# Canonical Ingest Ownership Spike

Status: proposed  
Priority: later  
Owner: unassigned  
Last reviewed: 2026-05-06

## Question

Should canonical ingest and index building move to Rust, or should Python/Node remain the owner while Rust only consumes prepared artifacts?

## Motivation

Rust is not bad at JSON; `serde` can model tolerant boundaries and strict normalized records well. The real tradeoff is between one canonical Rust domain model and the faster experimentation loop of Python or Node for messy vendor data and ML-oriented analysis.

## Prototype Scope

- Implement a small Rust ingest path over a representative Foundry PF2E fixture.
- Parse raw JSON tolerantly at the boundary.
- Normalize into typed runtime records.
- Write a minimal SQLite artifact matching the artifact contract spike.
- Compare implementation friction and drift risk against the current TypeScript ingest path and a plausible Python prep path.

## Do Not Mock

- messy nested Foundry JSON
- optional and missing fields
- record key/canonical name derivation
- semantic input construction
- schema writing
- source signature handling

## Outputs

- ingest ownership recommendation
- notes on where Python remains valuable for discovery/evaluation
- recommendation for canonical normalization ownership
- risks if prep and runtime remain split across languages

## Migration Dependency

This spike should follow the artifact contract and embedding spikes. If Rust query embeddings work well, moving canonical index build to Rust becomes more attractive.
