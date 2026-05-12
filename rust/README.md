# Rust Workspace

This workspace is the Rust migration foundation for PF2e Atlas. The Rust side is intended to become the owner of the deterministic core path: artifact validation, canonical ingest and index building, lookup/search runtime behavior, CLI workflows, and the Ratatui workbench.

TypeScript and Python may remain useful for exploratory analysis, parity comparison, and transitional tooling, but they are not the default long-term owners for canonical runtime artifacts.

## Layout

- `crates/atlas-domain`: shared Rust contracts and vocabulary.
- `crates/atlas-index`: index/artifact opening and validation.
- `crates/atlas-cli`: the local `atlas` command surface.

Add future crates only when their first real implementation slice lands.

## Validation

Run from this directory:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --workspace
```

The first CLI behavior is:

```bash
cargo run -p atlas-cli -- validate-index --index ../.cache/pf2e-index.sqlite --json
```

Current TypeScript-built indexes are expected to report a legacy-contract diagnostic until the Rust artifact contract is implemented by the index builder.

## Lint Policy

Use standard Clippy with warnings denied, plus `deny(unsafe_code)` in crates. Prefer typed errors in runtime code, but do not globally ban `unwrap` or `expect` during the scaffold phase; tests and short fixture setup often read more clearly with explicit panics. Add narrower lint denies later when a crate's runtime surface is stable enough to justify them.
