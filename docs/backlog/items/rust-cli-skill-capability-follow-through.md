# Rust CLI And Skill Capability Follow-Through

Status: proposed
Priority: soon
Owner: unassigned
Last reviewed: 2026-05-19

## Problem

The CLI plus first-party skill is the primary product surface. Product ideas that previously would have been phrased as transport or tool-schema enhancements should be evaluated as CLI command, JSON contract, setup, discovery, graph, or skill-guidance improvements.

Without a dedicated follow-up, useful query/discovery ideas can be lost when their old implementation wording is retired.

## Desired Outcome

Improve the CLI and skill around real task friction found during use.

Candidate areas:

- richer kind-specific preview facts in search results
- typo-tolerant or suggestion-oriented discovery without weakening strict `record resolve`
- pack/source listing if it proves useful for agents
- clearer command examples and command-choice rules in the skill package
- graph context guidance for common rule, condition, trait, and action questions
- better setup/readiness remediation text when an agent hits an artifact or embedding problem
- JSON examples for common batch workflows
- filter-discovery examples that reduce guessing around metadata, metrics, references, and traits

## Constraints

- Prefer small measured improvements tied to observed CLI or skill friction.
- Keep the product surface CLI-first; do not introduce a transport-specific compatibility layer.
- Keep durable behavior in runtime/search/index crates and command presentation in `atlas-cli`.

## Related

- [Runtime architecture](../../architecture/runtime.md)
- [ADR 0023: Rust CLI JSON contract](../../architecture/decisions/0023-rust-cli-json-contract.md)
- [ADR 0026: Rust CLI product surface](../../architecture/decisions/0026-rust-cli-product-surface.md)
