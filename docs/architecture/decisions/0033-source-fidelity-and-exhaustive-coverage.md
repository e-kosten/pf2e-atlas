# ADR 0033: Source Fidelity And Schema Discovery

Status: accepted; exhaustive coverage requirements superseded by [ADR 0037](./0037-source-schema-discovery.md) on 2026-10-05
Date: 2026-08-24

## Context

PF2e Atlas consumes a pinned Foundry/PF2e source whose durable contracts are distributed across Foundry document declarations, PF2e registrations and data models, templates, manifests, the observed corpus, and Atlas-generated relationships. The corpus alone cannot prove valid zero-count types or parent contexts, while broad raw JSON interpretation in downstream consumers loses ownership and creates inconsistent product behavior.

Atlas currently has no authentication or viewer authorization boundary and is primarily a GM tool. Source visibility and provenance still carry useful meaning, but treating them as an implicit authorization filter would hide authored information without an approved security model.

## Decision

`atlas-ingest` owns a versioned serialized-Source boundary. It uses a pinned source identity plus product-backed source envelopes and normalized models. Prepared Foundry Data is not the ingest contract. Original record bytes are deserialized once into one crate-private family-neutral recursive source tree before an ordinary JSON map can collapse authored structure. Ordered object members preserve duplicate keys with their distinct corresponding values, and arrays preserve order and multiplicity. Lookup distinguishes `Missing | Null | Value | Duplicate`; zero and false are meaningful values. Each family explicitly rejects, retains, or records typed unsupported duplicate evidence before a checked legacy JSON projection can serve unmigrated generic parsing. Omission from that projection is admitted only at an exact typed-owner boundary: the canonical spell system subtree (and typed image), excluding shared-content-owned `description.gm`, or an actual melee hazard child's damage map. Common record/provenance/content/taxonomy/mechanics/metrics parsing, NPC conversion, non-melee hazard fields, and non-family normalization remain projection consumers with fail-closed duplicate policy; projection is never their source-fidelity substitute. Duplicate identity and type dispatch discriminators always fail.

The first boundary version is `pf2e-serialized-source/v1`, pinned to PF2e system `6.12.4` at commit `4cbdaa37d6c33e9519561bae2c59a23e0288cbce`. Its hand-owned foundation dispatches NPC Actor sources and complete embedded Item source envelopes through the closed upstream Actor/Item discriminators, validates the exact NPC `items` parent relationship, and reports malformed shapes, unknown discriminators, and parent drift with record/source/JSON-path context. It retains the complete serialized tree without applying prepared-data defaults; the original raw JSON is exposed only for deliberate provenance/audit use. Canonical entity and occurrence interpretation is a later ingest phase, not an alternate adapter in this boundary.

Raw source snapshot identity is projection-independent. The v1 source signature is assembled from stable relative declared-input identities and raw content hashes before parsing, DTO, canonical, content, or enrichment projection can reject a record. Projection diagnostics and machine-local display text are not signed; identical source bytes at different roots have the same source identity.

Observed-source discovery follows the manifest-declared corpus. Registration-only possibilities and generated product records remain separate modeling concerns; discovery does not assign them to mandatory coverage owners.

Current product behavior is unauthenticated but the pinned base is not GM-complete: record default visibility, tooling/legacy routing, public-only content/reference participation, generated-affliction canonical/source-instance construction in `source_pipeline.rs` and `generated/afflictions/{mod.rs,records.rs}`, ingest embedding/report policy, FTS/search/filter keysets, graph/variants, discovery/metric catalogs, artifact validation, and downstream projections still contain classification-derived predicates. These do not establish a present privacy or security boundary.

Checkpoint A approved GM-complete behavior as the target. Useful authored information is not excluded solely because it is typed as public, GM, owner, private, hidden, internal, or because authorization is absent. Visibility, role, source kind, and provenance remain typed end to end. Future authenticated filtering requires a separate user-approved feature.

Product exclusions require a documented non-auth rationale and focused behavior tests. Valid rationales include implementation-only provenance, non-addressable container scaffolding, and avoiding duplicate ranking from copied capability prose. Classification alone is not a rationale.

Source parsers retain typed presence, unsupported-value, and duplicate policies. Focused parser, projection, and artifact tests protect these product behaviors. Raw JSON remains available for provenance and explicit offline schema discovery; runtime consumers use typed records.

Source discovery follows [ADR 0037](./0037-source-schema-discovery.md). Corpus snapshots inventory paths, types, counts, duplicate members, and examples. Baseline diffs reveal schema changes for review. They do not assign every source field to a model owner or omission decision, infer registration-only types, or certify exhaustive model completeness. The source-leaf ledgers, sealed receipt registry, and zero-unassigned gate are removed.

## Consequences

Family modeling remains driven by useful product behavior. New or changed observed shapes prompt inspection and a reviewed snapshot refresh; they do not automatically require a model expansion. Canonical family cutovers retain their typed owners and focused persistence and presentation tests.
