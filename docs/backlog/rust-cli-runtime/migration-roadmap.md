# Rust Runtime Roadmap

Status: active
Last reviewed: 2026-10-13

This roadmap groups durable runtime work. Use the linked backlog items for scope
and current status, and the architecture docs for ownership.

## Discovery

- [Search quality](../items/rust-search-quality-tuning.md): evaluate lexical and
  semantic relevance, ownership evidence and bounded candidate selection.
- [Query spelling](../items/rust-cli-typo-tolerant-discovery.md): offer useful
  suggestions while preserving strict record resolution and semantic discovery.
- [Filter catalog](../items/source-query-catalog-expansion.md): expose useful typed
  fields and scopes with consistent discovery and compilation.
- [Graph context](../items/rust-graph-context-deeper-local-graph.md): evaluate deeper
  related-record evidence where it improves reference workflows.

## Product Presentation

- [CLI and skill follow-through](../items/rust-cli-skill-capability-follow-through.md):
  address observed command, output and agent-guidance friction.
- [Responsive browser acceptance](../items/responsive-ui-validation.md): maintain
  device coverage for the shared browser components and interactions.
- [Encounter usability](../items/rust-web-encounter-runner-polish.md): prioritize
  construction, combat actions and readable participant information.

## Additional Consumers

- [Ratatui workbench](../items/rust-ratatui-workbench.md): build terminal workflows
  over the shared application and retrieval owners.
- [Derived tags](../items/rust-derived-tag-redesign.md): agree the ontology,
  assessment and editorial model before introducing runtime integration.

Current source loading, storage and retrieval follow
[ADR 0046](../../architecture/decisions/0046-source-backed-reference-artifact.md).
The [historical checklist](../history/rust-cli-runtime/migration-checklist.md)
records the migration baseline; it does not define new implementation work.
