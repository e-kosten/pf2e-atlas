# ADR 0035: Atomic Canonical Artifact

Status: proposed for Checkpoint B
Date: 2026-08-24

## Context

Source-faithful records require durable canonical entities, contextual occurrences, owned content, references, query projections, and complete hydration. Landing only a migration, writer, or partial reader would create a mixed old/new artifact contract and force downstream work to invent fallbacks.

## Decision

`atlas-index` owns the physical artifact schema, migrations, write model, complete canonical hydration, validation, inspection, and publication. Product-addressable activities, spellcasting entries, resource pools, owned content, and occurrence/reference identities use relational entities. Nested mechanics normally consumed with one parent may use deterministic Atlas-owned typed JSON. Facts that require independent filtering, joins, or aggregation use authoritative relational projections.

Canonical hydration has one owner: `atlas-index::read`. Search may consume hydrated records or explicit narrow read traits, but it may not create a second complete hydration path. Raw Foundry JSON is provenance and offline audit input, never a runtime fallback.

C1 is serialized, non-splittable, and atomic. One candidate must include:

- the migration and artifact contract/schema version bump;
- checked-in Diesel schema/models and required inventory;
- writers for canonical records, entities, occurrences, resources, content, references, metrics/facets, FTS, and embeddings;
- complete `atlas-index::read` hydration with stable IDs/order and foreign keys;
- canonical typed-JSON serialization/decoding and validation;
- atomic temporary-artifact publication;
- inspection, readiness, deep validation, CLI diagnostics, and corruption fixtures; and
- source-normalized versus artifact-hydrated deep equality over the canonical fixtures.

Old artifacts are rejected with actionable rebuild guidance. Compatibility adapters, partial migrations, empty-default hydration, and dual schema paths are not authorized.

Typed visibility/provenance remains stored independently from product retrieval disposition. Pinned-base retrieval still uses default-visible/public-only predicates and is not GM-complete. C1 must persist and validate the approved target disposition and rationale identity across FTS, embeddings, graph, discovery/metrics, inspection, and validation; useful authored information is eligible regardless of classification, and every retained exclusion requires non-auth product rationale and audit evidence.

## Consequences

Checkpoint C must approve the exact C1 commit and artifact hash before search, runtime, app, CLI, or UI consumers depend on the new artifact. Later search work may not amend canonical hydration under its own scope.
