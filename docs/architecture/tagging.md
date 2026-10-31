# Tagging architecture

Derived-tag development is deferred pending concrete needs unmet by native filters
and semantic search. [ADR0054](decisions/0054-derived-tag-deferral.md) records that
decision. The [research package](../research/derived-tags/README.md) owns historical
designs, vocabulary, evaluation and choices to revisit.

## Implemented surface

`atlas-tags` owns file model types, strict YAML parsing, corpus merging and
structural validation, exposed by `atlas tags validate`. Catalogs define valid tag
IDs; assignments and ontology suggestions are checked against their file contracts.
An assignment entry with an empty tag list records reviewed-empty; an absent entry
does not. Supplied evidence is checked structurally, and may be empty.

See the [model types](../../crates/atlas-tags/src/model.rs) and
[fixtures](../../crates/atlas-tags/tests/fixtures/) for exact file shapes.
[ADR0028](decisions/0028-rust-tagging-model.md) preserves the original design and
its status amendment.

Validation does not evaluate applicability against records, verify source-record
existence or evidence truth, classify records, assemble context, reconcile/import
proposals, or publish runtime tag filters. There is no artifact `record_tags`
projection or tagging predicate. Saved-list grouping tags are an independent
local-state feature.

## Boundaries for future work

Reopening requires a demonstrated retrieval gap and an explicit design decision.
Start with [design if revisited](../research/derived-tags/design-if-revisited.md),
then reconcile it with the current [architecture overview](overview.md).
Historical YAML, single-home grouping, minimal assessment provenance and unanimous
acceptance choices are not requirements for a replacement.

Keep record policies pure, reference schema and SQL in index, retrieval in search,
setup in runtime and composed workflows in app-service. Add authoritative tag
publication and query integration together if approved. Do not restore normalized
AtlasRecord, metric EAV, persisted RichDocument or Node heuristic derivation.
The existing validator does not authorize an assessment job or product integration.
