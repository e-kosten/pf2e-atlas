# Rust Runtime Checklist

Status: historical migration baseline with open follow-up links
Primary product surface: `atlas` CLI plus first-party agent skill
Architecture decisions: [ADR 0017](../../architecture/decisions/0017-rust-runtime-cli-first-migration.md), [ADR 0026](../../architecture/decisions/0026-rust-cli-product-surface.md)
Roadmap: [migration-roadmap.md](migration-roadmap.md)

This checklist records the original Rust runtime migration. Current storage and
retrieval policy is defined by ADR 0046 and the architecture docs; the coordinated
cutover remains open until its validation gates pass. Use the live backlog for
current follow-up status.

## Historical Completed Baseline

- [x] Rust workspace with `atlas-domain`, `atlas-record`, `atlas-ingest`, `atlas-index`, `atlas-embedding`, `atlas-search`, `atlas-runtime`, `atlas-cli`, and `atlas-sqlite-vec`.
- [x] Pinned Rust toolchain and tracked `Cargo.lock`.
- [x] Artifact contract `pf2e-atlas-artifact/v1`.
- [x] Deep and fast artifact validation.
- [x] Rust-owned Foundry source ingest and SQLite artifact writing.
- [x] Content documents, FTS projections, reference edges, remaster links, aliases, metrics, filter catalogs, embedding cache, and sqlite-vec vector index.
- [x] BGE small default embedding model with MiniLM support where explicitly selected.
- [x] `atlas setup` first-run install/repair flow.
- [x] `atlas record get` and `atlas record resolve`.
- [x] `atlas search` with filter-only, FTS, vector, and hybrid retrieval.
- [x] `atlas similar` for stored-embedding record-to-record retrieval with modest shared-reference/shared-trait evidence.
- [x] `atlas filters fields` and `atlas filters values`.
- [x] `atlas graph links|uses|variants|remaster` for bounded graph/product context.
- [x] Stable CLI JSON envelope and exit-code classes.
- [x] First-party PF2e Atlas CLI skill package with install and doctor commands.
- [x] Rust workspace promoted to repository root.

## Open Follow-Up

- [ ] [Source-backed artifact and ingest cutover](../items/typed-ingest-database-design.md).

- [ ] [Rust CLI and skill capability follow-through](../items/rust-cli-skill-capability-follow-through.md).
- [ ] [Rust search quality and retrieval weight tuning](../items/rust-search-quality-tuning.md).
- [ ] [Rust CLI kind preview facts](../items/rust-cli-kind-preview-facts.md).
- [ ] [Rust CLI typo tolerant discovery](../items/rust-cli-typo-tolerant-discovery.md).
- [ ] [Rust graph context deeper local graph](../items/rust-graph-context-deeper-local-graph.md).
- [ ] [Rust FTS tokenization and stemming exploration](../items/rust-fts-tokenization-stemming.md).
- [ ] [Rust Ratatui workbench](../items/rust-ratatui-workbench.md).
- [ ] [Rust derived-tag runtime and editorial redesign](../items/rust-derived-tag-redesign.md).

## Superseded Proposals

The old creature-level flag fix, field coverage ledger, promoted journal/table
subdocuments and metric convergence proposals are archived in
[backlog history](../history/done-and-superseded.md). Their intent is addressed
through the approved source-backed contract, rather than a parallel old model.

## Validation

Use the Rust gate from the repository root:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings -D clippy::dbg_macro
cargo clippy --workspace --lib --bins -- -D warnings \
  -D clippy::unwrap_used \
  -D clippy::expect_used \
  -D clippy::panic \
  -D clippy::unimplemented \
  -D clippy::todo \
  -D clippy::unreachable
cargo test --workspace
cargo build --workspace
```

Use Cargo's release profile for ingest and query performance comparisons:

```bash
cargo run --release -p atlas-dev -- source analyze --source vendor/pf2e --json
```

For command-surface smoke checks:

```bash
atlas setup --check --json
atlas index validate --json
atlas record resolve "Treat Wounds" --json
atlas search "low level healing spell" --limit 5 --json
atlas filters fields --json
atlas graph links actionspf2e:1kGNdIIhuglAjIp9 --json
atlas agent skills doctor --json
```

For root workspace cutover validation, include a structure check for Rust modules over roughly 700 lines. The remaining large files are pre-existing Rust implementation or integration-test owners; root promotion should not add behavior to those files beyond mechanical path and help-text updates.
