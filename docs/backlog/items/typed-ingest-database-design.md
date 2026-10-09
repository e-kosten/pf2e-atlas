# Typed ingest normalization and database design

Status: in_progress
Priority: after typed source loading
Owner: Codex
Last reviewed: 2026-10-09

## Implementation checkpoint

The local design study selected source-shaped authored models with focused Atlas
enrichment and relational query projections. Shared model ownership and exact
typed snapshots are implemented under ADR 0043; no product artifact, metric or UI
replacement follows automatically. Remaining work is product enrichment/content
policy, measured body/projection storage and coherent writer/reader/consumer
replacement. The design checkpoint remains open until implementation choices and
validation evidence are reflected in the artifact contract.

The shared content library is implemented under ADR 0044: ingest calls the
atlas-record parser directly, and source-backed HTML preparation returns HTML,
text, references, interaction parameters and diagnostics without another
persisted tree. The artifact and product consumers still use RichDocument.

ADR 0045 implements the database-independent source-backed envelope and consuming
typed ingest handoff. The library retains one DTO and source evidence, establishes
owned identity, prepares selected audience-filtered content, resolves reference
and relationship occurrences and exposes borrowed query/text views. The initial
[projection catalog and transition inventory](../../research/source-record-query-projections.md)
accounts for current filters and metrics without requiring a second family schema.
The [corpus probe evidence](../../research/source-record-enrichment.md) measures
exact snapshot preservation and enrichment cost; it
does not establish a physical storage format, product audience or search quality.

Before coherent adoption:

- Select product audience/implicit DC defaults, label/mechanics search projection,
  Embed expansion and asset resolution. The callable library requires explicit
  audience inputs and preserves authored labels without appended mechanics.
- Measure typed body/enrichment/cache storage and hydration. Cache identity must
  cover source/model, interpretation, localization and reference context.
- Bind the initial typed projections to an index-owned discoverable filter
  catalog and shared query compiler. Decide retain/rename/retire for functioning
  existing filters before cutover, and prove availability/negation and same-child
  SQL semantics. CLI CEL and structured UI filters can lower independently.
- Replace writer/readers and affected presentation/search consumers together;
  remove persisted RichDocument authority and duplicate macro/rendering policies.
- Correct faithful semantic text, model-specific pooling, scope-before-top-k and
  matched-unit identity; establish relevance with judged examples separately.
- Validate source-format selection, owned-node identity, partial fields, browser
  links/interactions and actual record routes. Foundry runtime preparation and
  dynamic gameplay execution remain separate behavior work.

## Intent

Perform a comprehensive ground-up review of Atlas normalization and database
design using the generated Foundry source DTOs, corpus evidence and required
query patterns. Typed loading supplies the input without committing to new
normalized records, storage layouts or metrics. This review is the design
checkpoint before adopting the new source stage into artifact construction.

## Required analysis

- Determine which source concepts become Atlas concepts. Consider family-specific
  facts, common facts, embedded documents, identities, relationships, provenance,
  raw fields and diagnostics. Preserve missing/null/invalid/not-applicable states
  where they matter; do not substitute zero or false for unavailable values.
- Reassess whether a generic metric model is justified at all. Its original
  source inputs were weakly modeled. Compare explicit typed family facts with
  derived query projections; distinguish authored and calculated values, units,
  context and applicability. Do not expand existing metric definitions by default.
- Design FTS text projections, search document boundaries, ranking and record
  mapping. Design semantic embedding units, contextual text and retrieval
  granularity together with hybrid retrieval and filtered search.
- Specify filters for scalar/multivalued fields and relationships, then evaluate
  relational tables, retained JSON and indexes against representative queries.
- Examine current query patterns and expected future product needs, including
  corpus scale, selectivity, result hydration and embedded-document retrieval.

## Deliverable and acceptance

Produce a concrete normalization/storage proposal, representative SQL queries,
index choices and measured query plans/performance. Record alternatives and
durable choices in architecture docs/ADRs. Describe raw/diagnostic retention and
what typed consumers can use for partially modeled documents. Define replacement
and artifact rebuild steps, validation comparisons and intended differences.

The current schema, metric abstraction and search projections are evidence to
evaluate, not constraints that must survive. Physical table layout and final enrichment/projection contracts remain to be
validated during implementation. Product UI and metrics
implementation follow the agreed design rather than preceding it.

## Inputs

- [ADR 0042: typed source loading](../../architecture/decisions/0042-typed-source-loading.md)
- [Source contract work](./rust-source-contract-generation.md)
- [Existing metric/source-fact convergence questions](./rust-side-data-metric-source-fact-convergence.md)
- [Existing artifact content-model questions](./rust-artifact-json-content-model-review.md)
- [Current artifact contract](../../architecture/artifact-contract.md)
