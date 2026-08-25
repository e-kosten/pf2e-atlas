# ADR 0033: Source Fidelity And Exhaustive Coverage

Status: proposed for Checkpoint B
Date: 2026-08-24

## Context

PF2e Atlas consumes a pinned Foundry/PF2e source whose durable contracts are distributed across Foundry document declarations, PF2e registrations and data models, templates, manifests, the observed corpus, and Atlas-generated relationships. The corpus alone cannot prove valid zero-count types or parent contexts, while broad raw JSON interpretation in downstream consumers loses ownership and creates inconsistent product behavior.

Atlas currently has no authentication or viewer authorization boundary and is primarily a GM tool. Source visibility and provenance still carry useful meaning, but treating them as an implicit authorization filter would hide authored information without an approved security model.

## Decision

`atlas-ingest` owns a versioned serialized-Source boundary. It uses a generated pinned source-contract/drift catalog plus hand-authored product-backed source envelopes and normalized models. Prepared Foundry Data is not the ingest contract. Where the pinned source permits the distinction, parsing preserves `Missing | Null | Value`; zero and false are meaningful values.

The first boundary version is `pf2e-serialized-source/v1`, pinned to PF2e system `6.12.4` at commit `4cbdaa37d6c33e9519561bae2c59a23e0288cbce`. Its hand-owned foundation dispatches NPC Actor sources and complete embedded Item source envelopes through the closed upstream Actor/Item discriminators, validates the exact NPC `items` parent relationship, and reports malformed shapes, unknown discriminators, and parent drift with record/source/JSON-path context. It retains the complete serialized tree without applying prepared-data defaults; the original raw JSON is exposed only for deliberate provenance/audit use. Canonical entity and occurrence interpretation is a later ingest phase, not an alternate adapter in this boundary.

Raw source snapshot identity is projection-independent. The v1 source signature is assembled from stable relative declared-input identities and raw content hashes before parsing, DTO, canonical, content, or enrichment projection can reject a record. Projection diagnostics and machine-local display text are not signed; identical source bytes at different roots have the same source identity.

Exhaustive discovery is the union of declarations, registrations/data models/templates, manifests, observed corpus, and Atlas-generated relationships. Each registry key includes document class, discriminator, role, exact parent context, and relationship path. Registration-only zero-count entries remain present, and observed-but-unregistered relationships remain conflicts until reviewed.

Every entry has an accountable current task or exact bounded future plan, owner, eventual acceptance, approval checkpoint, fixtures, and artifact/search/CLI/app/UI/runtime dispositions. Visibility, corpus count, generic deferral, `other`, `remaining`, or implicit parent ownership cannot satisfy assignment. The discovered and assigned sets must be identical and `unassigned_type_ids` must be empty.

Current product behavior is unauthenticated but the pinned base is not GM-complete: record default visibility, tooling/legacy routing, public-only content/reference participation, generated-affliction canonical/source-instance construction in `source_pipeline.rs` and `generated/afflictions/{mod.rs,records.rs}`, ingest embedding/report policy, FTS/search/filter keysets, graph/variants, discovery/metric catalogs, artifact validation, and downstream projections still contain classification-derived predicates. These do not establish a present privacy or security boundary.

Checkpoint A approved GM-complete behavior as the target. Useful authored information is not excluded solely because it is typed as public, GM, owner, private, hidden, internal, or because authorization is absent. Visibility, role, source kind, and provenance remain typed end to end. Future authenticated filtering requires a separate user-approved feature.

Every current exclusion requires an explicit non-auth product rationale, named owner, fixtures, validation, and audit checkpoint. Valid rationales include implementation-only provenance, non-addressable container scaffolding, and avoiding duplicate ranking from copied capability prose. Classification alone is not a rationale.

Real extractor owners declare source coverage. Local builds aggregate meaningful unknown/type-drift warnings. Strict corpus and source-refresh validation fails on new meaningful unknowns, type or presence drift, invalid parent contexts, coverage regression, registry/fixture drift, or lost ownership. Raw JSON remains available only for provenance and deliberate offline audit tooling; runtime consumers do not reparse it.

## Consequences

Creature-first implementation is bounded without narrowing discovery: the approved registry retains 313 exact assignments, including every non-creature, child, embedded, container, generated, and registration-only context.

Source updates require a reviewed registry and coverage diff. H1-H11 create separately approvable family plans after Checkpoint F. Every exact family plan must publish a field-level source-to-product disposition ledger, fixtures, exact task/path owners, non-auth product rationale, surface/runtime decisions, validation, and approval path. Hazard stealth/disable/routine/reset/action/affliction families and physical-item activation/uses/price/bulk/runes/subitem families are explicit examples, not catch-all categories. H12 independently re-derives the union, proves zero unassigned, and rejects a missing ledger/fixture or generic family substitution.
