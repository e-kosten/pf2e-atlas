# 0026 Rust CLI Product Surface

## Status

Accepted

## Context

The Rust runtime now owns the practical local workflow: setup, artifact construction, readiness validation, record lookup, strict resolution, ranked search, filter discovery, graph context, and first-party agent skill installation.

The CLI plus skill surface is sufficient for the repository's local agent and developer usage. Future product work should build on that surface and the Rust runtime crates instead of carrying an additional transport compatibility surface.

The interactive terminal and derived-tag/editorial workflows remain valuable, but they should be implemented as Rust-owned follow-up work after the CLI/runtime baseline.

## Decision

PF2e Atlas is a Rust CLI and first-party skill product.

- `atlas` is the primary user and agent interface.
- The first-party skill package is the agent guidance surface and is installed by `atlas agent skills`.
- Future interactive terminal work should be a Ratatui workbench over the Rust runtime crates.
- Future derived-tag work should be a Rust-owned redesign against record families, explicit metadata axes, typed discovery, and artifact ownership.
- Transport compatibility is not a planned product surface.

## Consequences

Product documentation should describe `atlas` commands and skill installation directly.

Architecture documentation should describe the Rust workspace as the current architecture. Historical rationale belongs in ADRs or backlog history, not in active product or contributor docs.

Backlog entries that preserve useful product intent should be reframed as Rust CLI, skill, Ratatui, search, graph, artifact, or derived-tag redesign work.

New durable behavior should route through the owning Rust crate:

- command presentation in `atlas-cli`
- path/setup policy in `atlas-runtime`
- retrieval orchestration in `atlas-search`
- artifact access and validation in `atlas-index`
- artifact writing in `atlas-index`
- schema descriptors in `atlas-artifact`
- normalized record/content models in `atlas-record`
