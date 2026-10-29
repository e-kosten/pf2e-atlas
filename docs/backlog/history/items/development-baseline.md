# Reusable development baseline

Status: done

PR85 and PR86 delivered the versioned baseline tooling and explicit backend selection.

Provide versioned validated executable/workspace seeds shared by main and linked
worktrees. Initialize missing private workspaces without replacing saved state;
share source/model caches and retain checkout-local compiler targets.

Acceptance: consistent live SQLite snapshot, atomic publication after validation,
independent writable copies, candidate readiness without implicit migration,
provenance/status commands, focused regression tests and real two-profile smoke.

See [the contributor workflow](../../../development/setup-and-preview.md#reusable-development-baseline) for commands and retention guidance.

The baseline manifest and existing private databases remain reusable across tooling
updates. Snapshot admission belongs to atlas-storage; atlas-dev owns orchestration
and the existing Rust test gate covers both.

Validation: full Rust fmt/Clippy/test/build gate, launcher/routing checks, native
full-corpus publication with deep validation and both vector profiles, independent
fresh clone readiness, existing main/worktree preservation, and isolated backend
readiness/record/assets smoke passed.
