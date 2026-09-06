# Rust Entity-Aware Query Facts And Filter Modeling

Status: proposed
Priority: later
Owner: unassigned
Last reviewed: 2026-09-04

## Problem

Canonical entity models are Atlas's authoritative semantic state, but the current query surface exposes much of its open scalar filtering through generic metric keys. The creature closeout correctly projects `CreatureRecord` one way through `project_creature_facts` into `MetricRow` values for artifact filtering and discovery, while creature presentation, encounter runtime, and hydration continue to read the canonical creature body. At the product boundary, however, `SearchFilterNode::Metric`, `MetricCompare`, and simple metric filters still identify facts with strings that are resolved through metric catalogs before SQL compilation.

Generic `record_metrics` storage is useful for heterogeneous SQL filtering and catalogs, but a generic metric key should not become the public semantic model for creatures or future record families. String keys alone cannot reliably communicate entity applicability, canonical field meaning, units, valid comparators, source presence, or provenance. Extending that shape family by family could make APIs and UI controls depend on storage vocabulary and tempt later consumers to reconstruct entity or runtime state from query rows.

Hazards and spells are likely evaluation points after the current creature closeout because they have coherent typed facts and important query dimensions. That observation sets research order only; this backlog item does not authorize a hazard/spell model, an implementation sequence, an artifact migration, or changes to the current creature contract.

## Desired Outcome

Evaluate a family-owned query-facts layer in which canonical entity models remain authoritative. A record family may own a typed projection such as `CreatureQueryFacts`, `HazardQueryFacts`, or `SpellQueryFacts` when its product queries justify one. Each projection would compile one way into a common query/filter intermediate representation, which `atlas-index` can lower to authoritative SQL. The artifact may retain generic metric rows and catalogs as internal storage or acceleration details.

The product-facing query contract should be:

- entity-aware, so field identity and applicability are explicit for each record family;
- typed, so values and source-presence/null behavior are not inferred from strings;
- unit-aware, so comparable values declare units or dimensions and incompatible comparisons fail closed;
- comparator-aware, so each field exposes only meaningful equality, ordering, range, set, text, or null operations;
- discoverable, so CLI, API, app-service, and UI can obtain labels, controls, operators, applicability, counts, and value/stat summaries from backend-owned metadata; and
- traceable, so every query fact identifies the canonical field and source/provenance evidence from which it was projected.

## Investigation

- Inventory current `MetricDefinition`, `MetricRow`, side-table, metadata-field, `SimpleSearchFilter`, `SearchFilterNode`, metric discovery, app filter-editor, and SQL compiler responsibilities. Identify which concepts are public semantics, common IR, catalog metadata, or internal storage.
- Map current creature query metrics back to exact `CreatureRecord` fields and provenance. Preserve `Missing | Null | Value`, numeric zero, unsupported values, collection identity/order where relevant, and field-specific diagnostics instead of treating absent metric rows as sufficient semantic evidence.
- Evaluate hazards and spells after the creature closeout as candidate family projections. Record which facts already have canonical typed owners, which remain deferred source fields, and which do not justify query promotion.
- Define cross-family query behavior. Distinguish genuinely shared concepts from same-named but incompatible family facts, specify union/intersection and mixed-kind behavior, and reject comparisons whose types, units, dimensions, or applicability do not align.
- Define the one-way common query/filter IR and its owner, including typed field identity, value shape, unit/dimension, allowed comparators, null/presence semantics, family applicability, canonical-field traceability, and lowering into the existing eligible-record keyset.
- Assess artifact and versioning implications. Determine whether the design changes only product/semantic contracts, changes projection or hydration compatibility requiring an `artifact_contract_version` bump, changes physical DDL requiring a `schema_version` bump, or changes the publication envelope requiring a `manifest_version` bump. Specify rebuild, compatibility, and validation behavior without dual semantic models or fallback decoders.
- Redesign filter discovery as needed so field catalogs and dynamic discovery expose entity-aware identities, labels, groups, controls, operators, units, applicable kinds, counts, numeric statistics, selected-field preservation, and self-excluding value scope without making the UI infer policy.
- Inventory every public or cross-layer consumer of string metric keys, including CLI/JSON filters, app-model DTOs, app-service lowering, URL state, generated TypeScript, and tests. Plan direct migration/removal of stringly semantic keys while allowing explicit internal storage keys to remain behind the compiler boundary.
- Compare generic metric storage with typed side-table or dedicated projection alternatives for correctness, performance, catalog generation, and cross-family queries. Storage selection must remain an index concern and must not determine the entity/API model.
- Define source-faithful evidence from the serialized Source DTO through canonical fields, typed query facts, common IR, artifact rows/catalogs, discovery, and filter results. Declarations, labels, destination names, and aggregate metric counts are not substitutes for exact value/type/presence, identity/order/multiplicity, and provenance observations through each declared owner.

## Constraints

- Canonical entity records remain the only semantic authority. Query facts, common IR, metric rows, catalogs, and SQL indexes are derived projections.
- Metrics must never hydrate or reconstruct presentation, encounter runtime, canonical entities, occurrences, resources, or other entity state, and they must not become a second semantic model.
- Presentation, FTS, embeddings, and runtime behavior must continue to consume canonical typed owners or their purpose-specific one-way projections, not query storage.
- Do not parse `raw_json`, metric-key segments, labels, or display strings to recover typed meaning. Do not move field policy or unit/comparator inference into CLI, API handlers, app-service adapters, or frontend code.
- Preserve source-faithful canonical facts and provenance even when a fact is not queryable. Query usefulness does not authorize dropping unsupported, missing/null, repeated, or provenance-only evidence.
- Do not add compatibility shims, parallel public filter models, dual writers, fallback decoders, or mixed old/new semantics unless a separately approved migration plan explicitly requires them.
- This item is research and future acceptance scope only. It does not authorize code, schema, artifact, generated DTO, UI, test, build, corpus, embedding, or family-model implementation.

## Acceptance Sketch

- A reviewed design names the owners and dependency direction for each family query-facts projection, the common query/filter IR, product discovery contracts, SQL lowering, storage rows/catalogs, and version gates.
- At least creature plus the evaluated hazard/spell candidates demonstrate that the design supports both family-specific fields and valid cross-family queries without erasing type, unit, comparator, applicability, or provenance distinctions.
- Every exposed query field has a stable typed identity, value shape, unit/dimension where applicable, allowed comparators, applicable families, canonical-field mapping, and discovery metadata. Invalid family, comparator, unit, or mixed-kind combinations fail with typed actionable errors.
- The design proves one-way flow from canonical entity to query facts to common IR/storage. Neither artifact hydration nor any presentation/runtime projection reads query metrics to recreate semantic state.
- The version decision is explicit and tested at the correct boundary: incompatible semantic/projection/hydration changes, physical DDL changes, and manifest-envelope changes use their existing distinct version owners and rebuild/validation rules.
- Catalog-backed and dynamic discovery tests cover entity-aware fields, controls, operators, units, kind applicability, selected fields, self-excluding counts, and stable ordering across single-family and cross-family scopes.
- Projection/compiler tests cover missing, null, zero, unsupported, repeated, incompatible-unit, invalid-comparator, unknown-field, and cross-family cases. Artifact validation proves exact query-row/catalog parity, while negative tests prove corrupted metrics cannot alter hydrated, presentation, or encounter-runtime state.
- Source-faithful fixtures and pinned-corpus evidence trace promoted facts through every declared final owner with exact value, type, presence, identity/order/multiplicity, canonical target, and provenance checks. Metric counts or destination labels alone cannot close coverage.
- Public stringly metric-key dependencies are removed or explicitly justified as internal serialization/storage identifiers behind typed boundaries; residue searches and generated-contract tests enforce the chosen end state.
- Architecture docs and any durable ADR are updated with the accepted ownership, one-way dependency, discovery, storage, migration, and versioning rules before implementation is reported complete.
- Hazard/spell implementation, if recommended, remains a separately planned and authorized follow-up after the creature closeout.

## Related

- [Architecture overview](../../architecture/overview.md)
- [Runtime architecture](../../architecture/runtime.md)
- [Artifact contract](../../architecture/artifact-contract.md)
- [Architecture decision index](../../architecture/decisions/README.md)
- [Rust Side Data And Metric Source Fact Convergence](./rust-side-data-metric-source-fact-convergence.md)
- [Rust Foundry Type Mechanics Parsers](../history/items/rust-foundry-type-mechanics-parsers.md)
- [Rust Web Filter UX Expansion](./rust-web-filter-ux-expansion.md)
- [Rust Web Filter State Policy Hardening](./rust-web-filter-state-policy-hardening.md)
