# Rust CLI Setup And Help Onboarding

Status: done

## Context

`atlas setup` behaved like a prerequisite/status check. It could report readiness while the default SQLite artifact was missing, which left standard users running `record get` or `record resolve` into an unavailable-index error. The CLI help text was also sparse, so users had to discover command guarantees, default paths, detail levels, and example keys from external context.

## Outcome

- `atlas setup` now runs the first-run install/repair flow for source, selected embedding target, artifact build, and final validation.
- `atlas setup --check` reports readiness and planned/blocked actions without mutating local runtime files.
- `atlas setup --offline` opts out of network source/model work and reports blocked actions when required local assets are missing.
- `atlas setup --no-embeddings` selects a record-only setup target for `record get`, `record resolve`, and filter-only operations.
- `atlas index validate` is symmetric with setup: default validation requires embedding/vector readiness, `--no-embeddings` validates the base artifact only, and `--embeddings-only` provides the focused vector diagnostic path.
- The separate `atlas index validate-vectors` command has been removed.
- CLI help now includes clearer setup, index, record, and search guidance with representative examples.

## Planning Artifact

The implementation plan was `scratch/plans/2026-05-17-rust-setup-full-install-and-cli-help.md`.
