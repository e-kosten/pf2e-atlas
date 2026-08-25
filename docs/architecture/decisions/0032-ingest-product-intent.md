# ADR 0032: Ingest Product Intent

## Status

Accepted.

## Context

PF2e Atlas keeps persisted Foundry raw JSON for provenance, parity debugging, and explicit offline audit tooling. The source corpus is broad and irregular enough that blindly mirroring every JSON field into Rust models, SQLite rows, or app DTOs would create a second Foundry schema inside Atlas without proving that the modeled data improves the product.

At the same time, leaving useful source structure in raw JSON pushes interpretation into later consumers. That makes search, CLI output, web presentation, encounter runtime behavior, agent workflows, and future API clients inconsistent or forces each surface to understand Foundry-specific JSON shapes.

Selective promotion does not permit selective discovery. The source-faithful contract separately requires every meaningful path and every document/type/role/parent-context tuple to have an explicit disposition and owner, including registration-only, container, generated, hidden, and provenance-only inputs.

The ingest path therefore needs an explicit product test for promoting source fields into typed Atlas facts.

## Decision

Ingest should not model source JSON because the field exists. It should model source facts when they improve at least one durable Atlas product surface:

- Search and discovery: filters, ranking signals, FTS or semantic text, facets, similar-record evidence, relationship discovery, and queryable mechanics.
- Record presentation and usability: readable detail surfaces, creature/stat-block presentation, embedded activities and spells, source-backed rich content, and product-oriented record summaries.
- Runtime play surfaces: encounter state, adjusted mechanics, conditions, variants, traits, action economy, hit points, resources, damage/healing workflows, and other table-time state projections.
- CLI and agent workflows: structured output that tools can consume without parsing prose or raw Foundry JSON, including prep workflows such as lists, encounters, and strict record resolution.
- Graph and reference behavior: explicit links, embedded references, `uses`/`used by`, remaster relationships, aliases, and generated records backed by source occurrences.
- Audit and data-quality feedback: offline reports that identify gaps, skipped source regions, and high-signal candidates for future typed projections.

Raw JSON audit tooling is allowed to scan broad source JSON because it is diagnostic input. Runtime lookup, search, filtering, presentation, encounter logic, and app-service DTOs should consume typed records, content documents, side tables, metrics, generated catalogs, and composed product views instead of reparsing raw JSON.

When a source field is promoted, the owning crate should match the product role:

- `atlas-ingest` owns Foundry source parsing, construction facts, normalization, and build-input preparation.
- `atlas-record` owns storage-agnostic normalized facts, content documents, metrics, mechanics views, and presentation-neutral source models.
- `atlas-index` owns durable SQLite schema, row validation, and storage/read models.
- `atlas-search` owns retrieval orchestration over indexed facts.
- `atlas-app-service` owns final product-facing compositions such as record surfaces and encounter-adjusted views.
- Frontends own interaction and rendering, not reinterpretation of Foundry source JSON.

Coverage dispositions may retain a field as owned content, derived projection, provenance-only input, ignored-with-non-auth-product-rationale, or an exact future family plan. Visibility, corpus absence, absent authorization, and generic deferral are not coverage dispositions. Current Atlas is unauthenticated but the pinned base is not GM-complete because default-visible/public-only routing still suppresses some product participation. Checkpoint A's target removes classification-only suppression; typed visibility/provenance does not establish a security boundary.

## Consequences

Source-field promotion work should begin by stating which product surface the field improves. For example, actor resource pools can support record presentation and encounter runtime; ritual DCs can support record presentation and mechanics projection; token names may support variant-family evidence, but should be weighed against their token-display semantics before becoming authoritative identity.

The audit command can reveal candidate gaps, but a high-frequency raw JSON path is not by itself a requirement to model that path. Conversely, a low-frequency path may still be worth modeling when it unlocks important record usability or runtime behavior.

Coverage labels in audit reports should mature toward declarations from the real extractor owners. Heuristic path-family maps are acceptable for early discovery, but they should not become a parallel source interpretation layer.

If a consumer needs to parse raw JSON at runtime to ship a feature, treat that as an ingest or projection gap unless the feature is explicitly diagnostic/debug tooling.
