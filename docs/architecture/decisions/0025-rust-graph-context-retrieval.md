# 0025 Rust Graph Context Retrieval

## Status

Accepted

## Context

The TypeScript MCP runtime exposes rule-context tools that combine rule name lookup with reference graph collection. Early Rust migration scaffolding mirrored that shape with rule-specific DTOs, but the Rust CLI product model has since settled on explicit record identification and result-set search as separate user intents.

For local agent workflows, the durable primitive is not "answer a rule question." It is: after a caller has identified a record key through `record get`, `record resolve`, or `search`, retrieve a bounded local reference neighborhood around that known key.

Normal search already supports relationship filters such as records that reference a key or are referenced by a key. Those filters answer result-set questions. They do not return a local context bundle with a seed record, neighbor records, edge evidence, counts, and truncation metadata.

## Decision

Rust graph retrieval exposes explicit graph product commands:

```bash
atlas graph links <record>
atlas graph uses <record>
atlas graph variants <record>
atlas graph remaster <record>
```

`graph links` retrieves a one-hop local graph context around one seed record. It accepts a canonical record key or strict resolvable record name, includes outgoing references by default, and includes backlinks only when the caller passes a positive backlink limit. `graph uses` is the backlinks-focused form for records that reference or use the seed. Both commands use the default public non-embedded reference graph policy and authored `reference_edges` only. `graph variants` exposes variant siblings/progressions from variant metadata, and `graph remaster` exposes legacy/remaster relationships from `remaster_links`.

Rust does not directly port `pf2e_collect_rule_question_context` in V1. The intended agent workflow is explicit and two-step: use `record` or `search` to identify the correct record, then call a graph command for the specific relationship view. A future rule-specific shortcut can be reconsidered only after real CLI usage shows that this explicit workflow is too costly.

Search relationship flags remain result-set filters. `atlas search --referenced-by` is not renamed or replaced by graph command wording. `--backlinks` is graph-command wording only.

V1 graph retrieval does not include name similarity, multi-hop traversal, graph scoring, semantic/vector search, answer synthesis, or visual graph output. Deeper local graph behavior is tracked separately in `docs/backlog/items/rust-graph-context-deeper-local-graph.md`.

The old rule-specific Rust DTOs are not accepted contracts. Implementation should delete `RuleContext*` and avoid compatibility aliases from `RuleGraph*` to generic names. V1 graph DTOs should live with the owning runtime crate, likely `atlas-search`, unless another Rust surface needs the exact same contract.

## Consequences

`atlas-index` owns read-only edge queries over `reference_edges` and SQL lowering of the reference graph policy.

`AtlasRetrievalService` owns graph context assembly: seed record loading, edge deduplication, deterministic edge ordering, unique-neighbor limits, retained-neighbor hydration, counts, and truncation metadata.

`atlas-cli` owns command arguments and presentation. JSON output follows the shared CLI envelope; human output stays concise and summary-oriented.

V1 graph retrieval accepts any seed record that direct key lookup can load, including non-default-visible records. Exposed edges and neighbor records are still constrained by the default graph policy, and hidden/private/internal edges do not affect counts or truncation.
