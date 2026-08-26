# ADR 0035: Atomic Canonical Artifact

Status: accepted at Checkpoint B; C1 candidate awaiting Checkpoint C
Date: 2026-08-24

## Context

Source-faithful records require durable canonical entities, contextual occurrences, owned content, references, query projections, and complete hydration. Landing only a migration, writer, or partial reader would create a mixed old/new artifact contract and force downstream work to invent fallbacks.

## Decision

`atlas-index` owns the physical artifact schema, migrations, write model, complete canonical hydration, validation, inspection, and publication. Product-addressable activities, spellcasting entries, resource pools, owned content, occurrence/reference identities, and typed creature grant/item-grant/linked-weapon/prepared-spell relationships use relational entities. Nested mechanics normally consumed with one parent may use deterministic Atlas-owned typed JSON. Facts that require independent filtering, joins, or aggregation use authoritative relational projections.

Canonical hydration has one owner: `atlas-index::read`. Search may consume hydrated records or explicit narrow read traits, but it may not create a second complete hydration path. Raw Foundry JSON is provenance and offline audit input, never a runtime fallback.

C1 is serialized, non-splittable, and atomic. Its v2 artifact unit includes:

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

Schema v2 stores creature bodies in deterministic typed JSON together with relational resources, entities, contextual occurrences, non-executing creature relationships, owned content, exclusions, and reference occurrences. Strict hydration and deep validation decode each body and require exact relational row sets across every authoritative column, including owners, parents, targets, source locators, lifecycle provenance, reference context, exclusions, typed content, and canonical metric facts; missing, extra, or valid-but-wrong foreign-key rows are corruption. Approved Stage B canonical identities are not rewritten when the source repeats a nested ID. Those repeated semantic IDs remain unchanged in the canonical body, while the relational primary/foreign-key locator includes authored order so every occurrence and content row remains independently durable.

Every v2 NPC row has exactly one required canonical creature body, every non-NPC row has none, and both all-record and by-key combined hydration reject missing or extra bodies. Inspection reports canonical creature-owned content separately from total artifact content.

Publication stages and syncs both files before changing the visible target. The adjacent v2 manifest is mandatory and binds `build.artifact_sha256`; an otherwise valid SQLite file or hard link without that manifest is unavailable. Readers and publishers coordinate through a persistent per-target OS lock. A reader takes the shared lock, opens one SQLite file generation, hashes that open file against the manifest, and opens and retains every SQLite connection for that reader before ending the acquisition boundary. Unix connections use the retained file descriptor. On shipped targets without a descriptor-backed SQLite path, including Windows x64, the reader retains the shared lock until its path-bound connections and verified file handle are dropped. It never verifies one path generation and later reopens another, and a publisher cannot swap the visible pair while a path-bound reader is active.

A publisher holds the exclusive lock across recovery, snapshot, replacement, verification, and cleanup. Fixed artifact-plus-manifest recovery backups make an interrupted replacement identifiable: a matching visible pair is committed and stale backups are removed; an invalid visible pair restores a matching backup pair when available or is removed before a new verified pair is installed. Ordinary failure restores both prior files, and first-publication failure removes both targets. The OS releases lock ownership after a process crash while the persistent coordination file preserves a single lock domain. Concurrent publishers therefore serialize, restoration cannot overwrite a later successful generation, and readers never accept a mixed or manifest-free generation while the stable user-facing SQLite and `manifest.json` paths remain unchanged.

Checkpoint C must approve the exact C1 commit and artifact hashes before search, runtime, app, CLI, or UI consumers depend on the new artifact. Later search work may not amend canonical hydration under its own scope.
