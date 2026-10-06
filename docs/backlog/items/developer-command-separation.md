# Developer command separation

Status: in_progress
Priority: next tooling cleanup
Owner: unassigned
Last reviewed: 2026-10-06

## Intended outcome

The installed `atlas` CLI serves user/agent workflows and operational maintenance.
A locally built `atlas-dev` Rust CLI serves development commands using the owning
Rust libraries. A private TypeScript package serves Foundry compiler discovery and
declaration research through its own npm commands. CI validates both developer
surfaces; release distributions include only the product CLI.

Rust developer commands do not start Node. Tools may exchange explicit JSON
artifacts when a later comparison needs both kinds of evidence.

## Implementation slices

1. The private `scripts/source-contracts` package uses strictly checked TypeScript
   implementation and tests, typed discovery/catalog outputs, a normal build and
   npm entry points. Its 24 fixtures and pinned extraction output remain stable.
2. Add `crates/atlas-dev` and move the existing ingest analysis, raw path audit and
   artifact inspection commands from `atlas index` into `atlas-dev source
   analyze`, `atlas-dev source audit-paths` and `atlas-dev index inspect`.
   Reuse atlas-ingest, atlas-runtime and atlas-index; remove old command routes.

These slices can be implemented independently once their existing dependencies
are available. Setup, index build/check/validate and product retrieval/filter
discovery remain in atlas. Keep authored-tag workflows out of this bounded move.

## Acceptance

- Rust developer commands preserve reports, input flags, path policy, failures
  and output channels; analysis writes no SQLite and inspection is read-only.
- TypeScript conversion preserves graph/catalog output, including unresolved
  constructs; all existing fixture tests pass and strict type checking runs in CI.
- No old command routes, parallel .mjs implementation or Rust-to-Node dispatcher
  remains. Rust developer commands run without Node installed.
- Product help/completions, architecture docs, contributor instructions and
  release selection agree with the three command owners.
- Both developer surfaces are validated in CI and excluded from published bundles.

## Related follow-up

- [Source-contract modeling experiment](./rust-source-contract-generation.md).
- [Selective integration recovery](./rust-integration-recovery.md): recover JSON
  schema snapshots, diffs and field sampling into atlas-ingest and atlas-dev in a
  separate slice. Use discovery evidence rather than field-owner receipts or a
  new runtime admission gate.
