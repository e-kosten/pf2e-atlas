# Rust Runtime Roadmap

Status: active
Last reviewed: 2026-05-19

PF2e Atlas is centered on a Rust local runtime with `atlas` as the command surface and the first-party skill as the local-agent guidance surface.

## Current Product Surface

- `atlas setup`: source, embedding cache, artifact build, repair, and readiness.
- `atlas record get`: exact key lookup.
- `atlas record resolve`: strict name and verified-alias resolution.
- `atlas search`: ranked text search and deterministic filter-only listing.
- `atlas filters fields|values`: schema and value discovery.
- `atlas graph get`: bounded one-hop graph context for known record keys.
- `atlas agent skills install|doctor`: first-party skill installation and inspection.

`atlas index analyze|build|check|inspect|validate` is the lower-level artifact and source diagnostic surface. It remains part of the CLI contract, but user-facing setup and repair guidance should lead with `atlas setup`.

## Roadmap

### CLI And Skill Follow-Through

- Improve skill examples as real task friction appears.
- Add family-specific preview facts where they help users and agents choose results faster.
- Add typo-tolerant discovery or suggestions without weakening strict record resolution.
- Keep setup and readiness errors actionable for local agents.

### Search And Retrieval Quality

- Tune FTS weights, retrieval windows, weighted fusion defaults, and tokenization choices with fixture-backed quality runs.
- Preserve one opinionated default search surface with advanced retrieval flags for diagnostics.

### Artifact And Ingest Completeness

- Audit Foundry JSON field coverage against Rust ingest.
- Add content subdocument treatment for journal pages and table results when the model is accepted.
- Converge overlapping side-data and metric source facts where a shared source interpretation is clear.

### Ratatui Workbench

- Build a Rust terminal workbench over the runtime crates.
- Cover search/browse, detail pages, filter exploration, graph context, and navigation.
- Add render tests and terminal capability abstractions.

### Derived Tags

- Redesign retained derived-tag concepts against record families, explicit source axes, typed discovery, and artifact ownership.
- Add Rust-owned runtime/editorial crates only when the accepted model is ready to implement.

## Non-Goals

- Do not add compatibility shims or parallel runtime owners.
- Do not add placeholder crates before a real implementation slice.
- Do not put durable retrieval or artifact semantics in CLI code.
- Do not port derived-tag/editorial concepts directly without a Rust model decision.
