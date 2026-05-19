# Rust CLI Runtime

This directory tracks the active Rust runtime work that remains after the CLI and first-party skill became the primary product surface.

## Active Documents

- [migration-checklist.md](./migration-checklist.md): current completion and follow-up checklist for Rust runtime work.
- [migration-roadmap.md](./migration-roadmap.md): forward-looking roadmap for CLI, skill, Ratatui, derived-tag, search, graph, and artifact improvements.

Historical spike notes and contract-mapping research live under [`docs/backlog/history/rust-cli-runtime/`](../history/rust-cli-runtime/).

## Current Focus

The active everyday product surface is:

- `atlas setup`
- `atlas record`
- `atlas search`
- `atlas filters`
- `atlas graph`
- `atlas agent skills`

`atlas index` remains available for diagnostics and contributor workflows, but normal users should usually reach for `atlas setup` first.

Open work should be framed as Rust CLI, skill, runtime, artifact, search, graph, Ratatui, or derived-tag redesign work.
