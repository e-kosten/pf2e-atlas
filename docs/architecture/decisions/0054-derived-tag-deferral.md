# Derived tags are deferred pending demonstrated retrieval gaps

Status: Accepted
Date: 2026-10-19
Partially supersedes: [ADR0028](0028-rust-tagging-model.md), for roadmap intent
and future design commitments; existing file validation contracts remain unchanged.

## Context

Derived tags were explored in the Node application, a canonical-concept model,
Rust model/corpus tooling and later ontology/assessment research. Improved semantic
search and source-backed native filtering may make the feature unnecessary. The
user prefers preserving the research and returning only when actual use reveals
needs that those surfaces cannot satisfy clearly.

The implemented subsystem provides file-owned definitions, assignments, strict
parsing and structural validation. It does not provide corpus-wide classification,
source-evidence verification or authoritative runtime tag filtering. Prior plans
and vocabulary experiments must not be mistaken for those capabilities.

## Decision

Defer ontology expansion, record assessment, editorial UI and runtime tag
integration. Preserve their important history, vocabulary, research and unresolved
choices in [the derived-tag research package](../../research/derived-tags/README.md).
Existing code and file contracts remain unchanged; removing them is separate work.

Reopening requires concrete user questions, expected/observed records and source
evidence, compared with current native filters and semantic/hybrid retrieval.
Start with the smallest concept that adds demonstrated value. Deferral is not a
claim that tags are universally unnecessary or that semantic search is complete.

ADR0028 remains provenance for the implemented file model. Its YAML choice,
single-home display grouping, minimal assessment provenance, negative-state policy
and unanimous automated acceptance are not binding design constraints on a future
replacement. The October discussion reopened those choices; any approved
implementation must explicitly reconcile model, workflow and architecture docs.
Do not restore Node heuristic derivation or run a full classification job from a
historical plan's instructions.

## Consequences

The backlog tracks a deferred capability, not a promised implementation. Current
architecture documents distinguish the small existing validator from conditional
future integration. Saved-list grouping tags remain an independent local-state
feature. Local research cleanup must verify snapshot coverage and omitted raw
artifacts before removal; this decision does not authorize deleting worktrees.
