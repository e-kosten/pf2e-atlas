# Reusable development baseline

Status: in_progress

Local implementation and operational acceptance are complete; integration is pending.

Provide versioned validated executable/workspace seeds shared by main and linked
worktrees. Initialize missing private workspaces without replacing saved state;
share source/model caches and retain checkout-local compiler targets.

Acceptance: consistent live SQLite snapshot, atomic publication after validation,
independent writable copies, candidate readiness without implicit migration,
provenance/status commands, focused regression tests and real two-profile smoke.

See [the contributor workflow](../../../CONTRIBUTING.md#reusable-development-baseline) for commands and retention guidance.

Local acceptance: 23 focused tests, validation-routing regressions, real baseline
publication/deep validation, both vector profiles, independent main/worktree
clones, repeat-init preservation and packaged backend smoke passed.
