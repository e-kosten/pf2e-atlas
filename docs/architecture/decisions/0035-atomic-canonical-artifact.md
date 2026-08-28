# ADR 0035: Atomic Canonical Artifact

Status: accepted at Checkpoint B; combined C1/C2/C2R/C2P/C2PR/C1C/C1R candidate awaits Checkpoint C
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

The unreleased v2/schema-2 layout directly persists canonical vector order for record metrics and already-modeled Activity and Spellcasting children. Metrics retain semantic-key uniqueness plus a non-null parent ordinal. Activity rows use `(record_key, ordinal)` as the internal storage identity because approved canonical parent-local `activity_id` values may repeat; `activity_id` remains unchanged, non-unique typed payload data and is not a public storage locator. Spellcasting rows retain parent/entry-ID identity and a unique parent ordinal. Both child tables cascade with record ownership and retain canonical typed payloads. All-record and by-key hydration order by parent and ordinal before grouping. The same layout stores closed typed visibility state/reason directly from `AtlasRecord.visibility`, while the role/disposition/rationale routing tuple and its derived `is_default_visible` projection remain a separate contract. Neither contract is reconstructed from the other.

## Consequences

Schema v2 stores creature bodies in deterministic typed JSON together with relational resources, entities, contextual occurrences, non-executing creature relationships, owned content, exclusions, reference occurrences, and ordered record mechanics children. Strict hydration and deep validation decode each body and mechanics payload and require exact relational row sets across every authoritative column, including owners, parents, targets, source locators, lifecycle provenance, reference context, exclusions, typed content, canonical metric facts, child IDs, and vector order; missing, extra, reordered, reparented, or valid-but-wrong foreign-key rows are corruption. Approved Stage B canonical identities are not rewritten when the source repeats a nested ID. Those repeated semantic IDs remain unchanged in the canonical body, while the relational primary/foreign-key locator includes authored order so every occurrence and content row remains independently durable.

Every v2 NPC row has exactly one required canonical creature body, every non-NPC row has none, and both all-record and by-key combined hydration reject missing or extra bodies. Inspection reports canonical creature-owned content separately from total artifact content.

Publication stages and syncs both files before changing the visible target. The adjacent v2 manifest is mandatory and binds `build.artifact_sha256`; an otherwise valid SQLite file or hard link without that manifest is unavailable. Readers and publishers coordinate through a persistent per-target OS lock. A reader takes the shared lock, opens and hashes the visible SQLite file against the manifest, materializes or verifies an adjacent immutable generation snapshot named by that digest, and opens and retains every SQLite connection on the snapshot before ending the acquisition boundary. It then releases the lock regardless of platform. Long-lived readers remain usable on their verified old generation while a publisher installs a new visible pair, and new readers bind to the new digest-named generation. The snapshot is an internal reader-lifecycle cache, not a second artifact publication contract or runtime refresh path.

A publisher acquires the exclusive lock with a five-second deadline and actionable retry failure, then holds it across recovery, snapshot, replacement, verification, and cleanup. Fixed artifact-plus-manifest recovery backups make an interrupted replacement identifiable: a matching visible pair is committed and stale backups are removed; an invalid visible pair restores a matching backup pair when available or is removed before a new verified pair is installed. Ordinary failure restores both prior files, and first-publication failure removes both targets. The OS releases lock ownership after a process crash while the persistent coordination file preserves a single lock domain. Concurrent publishers therefore serialize or fail within the documented bound, restoration cannot overwrite a later successful generation, and readers never accept a mixed or manifest-free generation while the stable user-facing SQLite and `manifest.json` paths remain unchanged. Successful publication and final reader drop clean obsolete generation snapshots without deleting a generation that Windows still reports in use.

The provisional C1 commit establishes the artifact semantics. C2 adds only the
private validation-pipeline tooling described above it in the approved task graph;
it does not approve C1 or change the artifact contract. C2R restores the accepted
pre-localization strict-audit observation from already captured raw records while
leaving localized product normalization unchanged, and requires complete detailed
audit evidence to be atomically persisted and checksum-bound before either PASS
or FAIL returns. C2P composes validation over one private verified-generation
handle and one live deep-validation receipt per artifact mode. It preserves every
independent atomic SHA check and the required generation copy while eliminating
only repeated validation-side reader/scanning work; neither capability is
serialized or available to runtime/product consumers. C2PR resolves private
validation selectors through the existing typed embedding catalog before source
work, retains canonical artifact-metadata checks after publication, and
atomically preserves typed failure/partial-timing evidence while reusing already
trusted generation digests. It changes neither embedding nor artifact semantics.
C1C corrects only the unreleased artifact persistence and hydration contract for already-modeled ordered mechanics and independent typed visibility, without changing Stage B canonical semantics. C1R then remediates only the independently reproduced residual C1 findings on top of C2,
C2R, C2P, C2PR, and an independently accepted C1C. Checkpoint C must approve the exact combined
C1/C2/C2R/C2P/C2PR/C1C/C1R commit chain and final independently reproduced artifact hashes
before search, runtime, app, CLI, or UI consumers depend on the new artifact.
Later search work may not amend canonical hydration under its own scope.

The provisional-C1 versus combined optimized-head legacy/new matrix is authorized
once to establish trust in the consolidation. After that independent equivalence
PASS, it retires immediately: permanent validation consists only of the
consolidated fast, focused, exhaustive, and reviewer tiers, with exhaustive work
triggered by relevant changes. The migration matrix is not added to normal CI,
future candidate acceptance, or Checkpoint C.
