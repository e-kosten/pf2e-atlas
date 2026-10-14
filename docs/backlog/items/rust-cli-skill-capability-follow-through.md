# CLI and Skill Usability

Status: proposed
Priority: soon
Owner: unassigned
Last reviewed: 2026-10-13

Improve the CLI and first-party skill around observed lookup, discovery, output
and readiness friction. Prioritize changes that help people and agents choose
commands and interpret results reliably.

Candidate work includes concise kind-specific search facts, useful miss handling,
pack/source discovery, clearer command examples, graph-context guidance,
actionable readiness remediation and batch JSON examples. Filter guidance should
use the typed catalog and distinguish field availability from matching values.

Keep each improvement tied to a concrete task and evaluate its result. Shared
product behavior belongs to its runtime, search or application owner; CLI
presentation belongs in atlas-cli. Coordinate overlapping browser discovery
needs through the shared contracts.

Related: [kind preview facts](rust-cli-kind-preview-facts.md),
[query spelling](rust-cli-typo-tolerant-discovery.md),
[runtime architecture](../../architecture/runtime.md).
