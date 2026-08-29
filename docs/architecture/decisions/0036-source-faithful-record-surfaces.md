# ADR 0036: Source-Faithful Record Surfaces

Status: proposed for Checkpoint B
Date: 2026-08-24

## Context

Atlas needs coherent record selection, preparation, agent, and encounter views without duplicating canonical semantics across CLI, app DTOs, and frontend code. It also needs to preserve authored content and runtime adjustments without keeping the current creature fallback as a permanent second record.

## Decision

`atlas-app-service` composes three app profiles through generated `atlas-app-model` DTOs:

- `search_compact` for identity, teaser, scan facts, compact traits, and source;
- `record_detail` for complete scan-first awareness, skills, defenses, movement, strikes, spellcasting, abilities, resources/gear, owned content, relationships, and provenance; and
- `encounter_participant` for participant state, final values and explanations, conditions, turn/action state, usable activities, resources, and source-backed capabilities.

`atlas-web` remains transport. `web/atlas-ui` uses Ant Design for generic application controls and PF2e-specific components for record/runtime layouts. It renders generated semantics and hosts feature-owned interaction slots; it does not parse Foundry JSON or prose, infer mechanics, or reconcile static and runtime facts.

ADR 0023 remains authoritative for CLI JSON. CLI record-bearing commands share one `atlas-record::RecordJson`: a genuinely common flattened base plus a mandatory tagged entity presentation. The first family-specific body is creature and directly exposes typed defenses, awareness, skills, movement, resources, activities, and spellcasting. Rich content remains supplementary. Non-creature families use an explicit registry-bound `unmigrated` boundary only until their H-family plan lands. The earlier additive generic-section direction is superseded; this unreleased contract is replaced directly with no compatibility shim, dual serializer, or feature switch. App DTOs and a parallel CLI semantic model remain forbidden.

`RichDocument` remains the once-parsed authored content tree. Record, entity, occurrence, child, content, and section targets remain addressable. The default child embedding policy remains overflow-only. Copied embedded capability prose stays reachable through its owner but is excluded from default ranking to prevent duplicate-result flooding, not because of authorization.

Current CLI/app/UI behavior is unauthenticated but inherits pinned-base default-visible/public-only routing and is not GM-complete. The approved target preserves typed visibility, role, and provenance as metadata while preventing classification-only suppression of useful authored information; no present security boundary is claimed. D2 produces CLI JSON/text and agent behavior, then D3 audits only the available source/artifact/search/CLI/agent surfaces. E3 separately verifies generated app DTOs/composition/transport. F1/F2 produce record and encounter UI, F3 audits the completed browser/static/runtime matrix, Checkpoint E owns human visual approval, and G2 verifies the final staged evidence and cutover residue.

A4 screenshots are diagnostic only. F3 automation runs only after F2 and must pass the complete hashed fixture/viewport/theme/static/runtime matrix, including rejection of classification-only suppression; only Checkpoint E provides human visual approval. Creature fallback removal is forbidden before that approval, and G2 later verifies the complete evidence chain.

## Consequences

Search, CLI, app, and UI can differ in projection shape while deriving from one canonical semantic record. Runtime final values replace duplicate base display values. Non-creature families keep exact H1-H11 ownership and require separate plans and approvals; each plan must include a field-level disposition ledger, fixtures, exact owners/rationales, surface/runtime decisions, validation, and approval path, and H12 rejects missing or generic packages.
