# Architecture decisions

Start with the [architecture overview](../overview.md) for current ownership,
then read the decisions relevant to the change. [Runtime](../runtime.md) and the
[artifact contract](../artifact-contract.md) describe the current operating model.

ADRs record rationale and accepted boundaries. Some are amended in part: read
their status and successor links before applying historical examples. Updating
implementation status does not require a new ADR; a new durable decision does.

## Active decisions

### Product and contracts

- [0019: CLI record/search intent](0019-rust-cli-search-record-surface.md)
- [0023: CLI JSON and batch results](0023-rust-cli-json-contract.md)
- [0026: Rust product surface](0026-rust-cli-product-surface.md)
- [0029: Web/app boundary and browser compatibility](0029-local-web-app-boundary.md)
- [0064: Optional web distribution](0064-optional-web-distribution.md)
- [0033: Developer command surfaces](0033-developer-command-surfaces.md)
- [0047: Semantic record presentation](0047-semantic-record-presentation.md)
- [0058: Browser routing](0058-browser-routing.md)

### Workspace and workflows

- [0021: Runtime/index/retrieval ownership](0021-rust-runtime-index-and-retrieval-boundaries.md)
- [0022: Artifact policy ownership](0022-rust-artifact-policy-ownership.md)
- [0048: Atomic mutable workflows](0048-atomic-mutable-workflows.md)
- [0051: Persistent workspace and bounded sessions](0051-persistent-workspace.md)
- [0052: Pinned reference source](0052-pinned-reference-source.md)
- [0055: Condition evaluation](0055-encounter-condition-evaluation.md)
- [0061: Encounter recovery](0061-encounter-recovery.md)

### Search and relationships

- [0025: Graph context](0025-rust-graph-context-retrieval.md)
- [0046: Source-backed reference artifact](0046-source-backed-reference-artifact.md)
- [0049: Search lifetimes and pages](0049-search-lifetimes-and-pages.md)
- [0050: Selectable search profiles](0050-selectable-search-profiles.md)
- [0053: Field-aware fusion](0053-field-aware-search-fusion.md)
- [0056: Trait reference entities](0056-trait-reference-entities.md)
- [0057: Relationship-scoped search](0057-relationship-scoped-search.md)
- [0059: Configurable Similar](0059-configurable-similarity.md)
- [0060: Core sections and scopes](0060-core-sections-and-search-scope.md)
- [0062: Exact text-neighbor reuse](0062-exact-text-neighbor-reuse.md)
- [0063: Operation-local eligibility prototype](0063-request-local-configured-eligibility.md)

### Source models and preparation

- [0027: Runtime lint policy](0027-rust-runtime-lint-policy.md)
- [0032: Product intent for derived projections](0032-ingest-product-intent.md)
- [0034: Generation layout](0034-source-generation-layout.md)
- [0035: Pre-default source values](0035-source-value-generation-policy.md)
- [0036: Recursive unions](0036-recursive-source-unions.md)
- [0037: Open/indexed values](0037-open-and-indexed-source-values.md)
- [0038: Templates and generic names](0038-source-templates-and-generic-names.md)
- [0039: Authored rule inputs](0039-authored-rule-inputs.md)
- [0040: Authored document inputs](0040-authored-document-inputs.md)
- [0041: Admission and field retention](0041-source-admission-and-field-retention.md)
- [0042: Typed source loading](0042-typed-source-loading.md)
- [0043: Shared model and snapshots](0043-shared-foundry-model-and-snapshots.md)
- [0044: Shared content interpretation](0044-shared-source-content-interpretation.md)
- [0045: Source-backed enrichment](0045-source-backed-record-enrichment.md)

### Deferred tagging

- [0028: Historical file model; product integration deferred](0028-rust-tagging-model.md)
- [0054: Deferral pending demonstrated retrieval gaps](0054-derived-tag-deferral.md)

## Historical decisions

The [historical catalog](history/README.md) holds retired TypeScript/MCP/TUI
decisions, Rust migration sequencing and fully replaced Rust designs. Retain
their IDs and rationale when retiring a decision; repair incoming links and
identify the successor. Partly amended decisions stay above while their retained
boundaries still apply.
