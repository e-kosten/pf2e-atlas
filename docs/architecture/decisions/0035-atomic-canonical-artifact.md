# ADR 0035: Atomic Canonical Artifact

Status: accepted at Checkpoint B; combined C1/C2/C2R/C2P/C2PR/C1C/C1R candidate awaits Checkpoint C
Date: 2026-08-24

## Context

Source-faithful records require durable canonical entities, contextual occurrences, owned content, references, query projections, and complete hydration. Landing only a migration, writer, or partial reader would create a mixed old/new artifact contract and force downstream work to invent fallbacks.

## Decision

`atlas-index` owns the physical artifact schema, migrations, write model, complete canonical hydration, validation, inspection, and publication. Product-addressable activities, spellcasting entries, resource pools, owned content, occurrence/reference identities, and typed creature grant/item-grant/linked-weapon/prepared-spell relationships use relational entities. Nested mechanics normally consumed with one parent may use deterministic Atlas-owned typed JSON. Facts that require independent filtering, joins, or aggregation use authoritative relational projections.

Canonical hydration has one owner: `atlas-index::read`. Search may consume hydrated records or explicit narrow read traits, but it may not create a second complete hydration path. Raw Foundry JSON is provenance and offline audit input, never a runtime fallback.

C1 is serialized, non-splittable, and atomic. Its artifact unit includes:

- the migration and artifact contract/schema version bump;
- checked-in Diesel schema/models and required inventory;
- writers for canonical records, entities, occurrences, resources, content, references, metrics/facets, FTS, and embeddings;
- complete `atlas-index::read` hydration with stable IDs/order and foreign keys;
- canonical typed-JSON serialization/decoding and validation;
- atomic temporary-artifact publication;
- inspection, readiness, deep validation, CLI diagnostics, and corruption fixtures; and
- source-normalized versus artifact-hydrated deep equality over the canonical fixtures.

Old artifacts are rejected with actionable rebuild guidance. Compatibility adapters, partial migrations, empty-default hydration, and dual schema paths are not authorized.

Three code-owned versions describe compatibility. `artifact_contract_version` is the umbrella for incompatible canonical codec, deterministic projection, complete hydration, and artifact/publication semantics; `schema_version` identifies physical SQLite DDL; and `manifest_version` identifies the adjacent envelope shape and interpretation. There are no component projection, builder, completion, or SQLite `user_version` mirrors. Reader acquisition cheaply requires exact manifest and SQLite contract/schema stamps. A deterministic merge-base source-policy guard maps schema, contract, and envelope owners to those versions, reports every affected class, and is a required pull-request CI status; local hooks provide only advisory, bypassable early feedback. Small compatibility fixtures and unit/integration tests are the primary semantic proof.

Typed visibility/provenance remains stored independently from product retrieval disposition. Pinned-base retrieval still uses default-visible/public-only predicates and is not GM-complete. C1 must persist and validate the approved target disposition and rationale identity across FTS, embeddings, graph, discovery/metrics, inspection, and validation; useful authored information is eligible regardless of classification, and every retained exclusion requires non-auth product rationale and audit evidence.

The unreleased v2/schema-2 layout directly persists canonical vector order for record metrics and already-modeled Activity and Spellcasting children. Metrics retain semantic-key uniqueness plus a non-null parent ordinal. Activity rows use `(record_key, ordinal)` as the internal storage identity because approved canonical parent-local `activity_id` values may repeat; `activity_id` remains unchanged, non-unique typed payload data and is not a public storage locator. Spellcasting rows retain parent/entry-ID identity and a unique parent ordinal. Both child tables cascade with record ownership and retain canonical typed payloads. All-record and by-key hydration order by parent and ordinal before grouping. The same layout stores closed typed visibility state/reason directly from `AtlasRecord.visibility`, while the role/disposition/rationale routing tuple and its derived `is_default_visible` projection remain a separate contract. Neither contract is reconstructed from the other.

The Activity identity decision is grounded in the pinned Shobhad source projection, not only a synthetic duplicate. `pfs-season-3-bestiary:EB00f6ADElWInuix` (Shobhad Hunter) has source `system.items[10]` and `system.items[11]` activities with sorts `1100000` and `1200000`; both project to canonical `RecordMechanics.activities[6]` and `[7]` with `activity_id=AMqdiX2GpuYvsQOp`, label `Four-Armed`, and kind `Other`. `strength-of-thousands-bestiary:RJKVH3fxPEiTCwt5` (Shobhad Sniper) likewise has source `system.items[5]` and `[6]` activities with sorts `600000` and `700000`; both project to canonical positions `[2]` and `[3]` with `activity_id=lLXZFku1wFZoAPdz`, label `Four-Armed`, and kind `Other`. These are same-record duplicate payload identifiers. They are neither public record identities nor cross-record references, so storage must preserve the payload value while locating each ordered row only by `(record_key, ordinal)`.

## Consequences

Schema v2 stores creature bodies in deterministic typed JSON together with relational resources, entities, contextual occurrences, non-executing creature relationships, owned content, exclusions, reference occurrences, and ordered record mechanics children. Strict hydration and deep validation decode each body and mechanics payload and require exact relational row sets across every authoritative column, including owners, parents, targets, source locators, lifecycle provenance, reference context, exclusions, typed content, canonical metric facts, child IDs, and vector order; missing, extra, reordered, reparented, or valid-but-wrong foreign-key rows are corruption. Approved Stage B canonical identities are not rewritten when the source repeats a nested ID. Those repeated semantic IDs remain unchanged in the canonical body, while the relational primary/foreign-key locator includes authored order so every occurrence and content row remains independently durable.

Every v2 NPC row has exactly one required canonical creature body, every non-NPC row has none, and both all-record and by-key combined hydration reject missing or extra bodies. Inspection reports canonical creature-owned content separately from total artifact content.

Publication stages and syncs both files before changing the visible target. The adjacent v3 manifest is mandatory and binds `build.artifact_sha256`; an otherwise valid SQLite file or hard link without that manifest is unavailable. Readers and publishers coordinate through a persistent per-target OS lock. A reader takes the shared lock, opens the visible SQLite file and manifest, materializes an absent adjacent immutable generation snapshot named by the manifest digest, and opens and retains every SQLite connection on that snapshot before ending the acquisition boundary. A newly copied generation is synced and hashed exactly once at that copy boundary; an existing locally published generation is opened without a whole-file hash. The reader then performs the cheap exact contract/schema stamp check and releases the lock regardless of platform. Long-lived readers remain usable on their old generation while a publisher installs a new visible pair, and new readers bind to the new digest-named generation. The snapshot is an internal reader-lifecycle cache, not a second artifact publication contract or runtime refresh path.

A publisher acquires the exclusive lock with a five-second deadline and actionable retry failure, then holds it across recovery, snapshot, replacement, verification, and cleanup. Fixed artifact-plus-manifest recovery backups make an interrupted replacement identifiable: a matching visible pair is committed and stale backups are removed; an invalid visible pair restores a matching backup pair when available or is removed before a new verified pair is installed. Ordinary failure restores both prior files, and first-publication failure removes both targets. The OS releases lock ownership after a process crash while the persistent coordination file preserves a single lock domain. Concurrent publishers therefore serialize or fail within the documented bound, restoration cannot overwrite a later successful generation, and readers never accept a mixed or manifest-free generation while the stable user-facing SQLite and `manifest.json` paths remain unchanged. Successful publication and final reader drop clean obsolete generation snapshots without deleting a generation that Windows still reports in use.

Writer publication uses one non-serializable `ArtifactPublicationReceipt`. After SQLite handles close and the staged file is synced, companion-free, and sealed, a cheap metadata compatibility check and one producer SHA-256 pass operate through one retained file identity. The receipt binds that handle, its portable Unix device/inode or Windows volume/file-index identity, path, intended target, byte count, modification token, digest, and compatibility result. Ingest may carry the receipt and use its digest for the manifest but owns no identity or validation policy. Publication consumes the receipt, rejects drift, and performs no second staged or renamed-file hash. It performs one ordinary byte copy into an absent digest-named generation, syncs and hashes that copy exactly once, reuses an existing local generation without reopening or rehashing it, and rejects matching source/generation identities; hard links, reflinks, and other shared-mutation aliases are forbidden. Recovery hashes a candidate visible or backup pair only at that recovery boundary and reuses the result when snapshotting the prior pair.

Ordinary all-record and keyed hydration decode and validate the rows they request, including required typed bodies and relational children, but do not launch global canonical coherence. Malformed requested values continue to fail with typed decode/query errors. The writer enables SQLite foreign-key enforcement before its transaction. Full foreign-key, all-row projection equality, FTS/embedding coverage, integrity, and canonical-coherence scans remain explicit supplementary validation tooling and builder/CI tests; they are not ordinary open, setup/readiness, publication, inspection, or keyed hydration preflights.

The provisional C1 commit establishes the artifact semantics. C2 adds only the
private validation-pipeline tooling described above it in the approved task graph;
it does not approve C1 or change the artifact contract. C2R restores the accepted
pre-localization strict-audit observation from already captured raw records while
leaving localized product normalization unchanged, and requires complete detailed
audit evidence to be atomically persisted and checksum-bound before either PASS
or FAIL returns. C2P composes explicit validation over one private generation-bound
handle and one live deep-validation result per artifact mode. It retains the
required generation copy while eliminating repeated validation-side reader and
scanning work; neither capability is
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
