# Compiler-based source contracts and trait catalog

Status: in_progress
Priority: next source-modeling experiment
Owner: unassigned
Last reviewed: 2026-10-06

## Problem and evidence

The source-modeling work on integration aims to define the complete pinned source
portfolio before post-parser storage, metrics and UI expansion. Repeated manual
translation of upstream TypeScript risks duplicated shared shapes and drift.
Foundry trait labels, descriptions and tag catalogs are also missing from Atlas's
product vocabulary, despite trait identifiers already supporting filtering.

The comparison at PR18 tested ts2rs, quicktype and JSON Schema plus typify against
PF2e 6.12.4 at `4cbdaa37d6c33e9519561bae2c59a23e0288cbce`. None produced a safe
drop-in full source model. ts2rs emitted empty structs that accepted 5,263 selected
occurrences and discarded all their fields. Reduced generated Rust also exposed
presence, discriminator, numeric and keyed-duplicate losses. The upstream
environment was partial; strict project type-check failures alone are not tool
defect evidence.

TypeScript compiler extraction did resolve inherited envelope/system fields,
shared type identities and family-forbidden fields for four physical families.
Extraction is demonstrated; reliable full Rust generation remains unproven.

## Proposed slices

1. Produce deterministic compiler-based extraction for the persisted source
   portfolio: all pack document kinds, registered Actor and Item families,
   embedded source shapes and built-in rule schemas. Preserve named references,
   recursion, optional/null/never distinctions and source locations. Diagnose
   unresolved types rather than emitting empty structs or arbitrary JSON.
2. Extract upstream trait and other-tag catalogs with identifiers, labels,
   localization keys, authored description text and catalog/family membership.
   Preserve missing metadata and parameterized variants. Keep this independent
   of database construction, API/UI behavior and Atlas authored tags.
3. Test bounded Rust generation for a shared physical component and equipment
   refinement using existing source-presence, numeric, ordered-member and
   diagnostic primitives. Compare meaningful typed output against current
   parsers, pinned corpus and adversarial fixtures before deciding on adoption.
4. Adopt only proven useful generation through finished source-only replacements;
   remove corresponding duplicate manual owners in each replacement slice.
   Retain extraction as discovery tooling if generation adds excessive complexity.

## Constraints and completion

Ownership stays with atlas-ingest and supporting offline developer tooling;
ordinary Rust/runtime consumers need no TypeScript runtime. Use upstream Source
contracts plus observed legacy data; prepared Foundry Data is a separate shape.
Share payloads when their semantics and fidelity match. Do not force raw source
types to become frontend/API models; continue generating TS from Rust API contracts.

Trait vocabulary can remain strings with a generated metadata catalog; final
enum/open-string policies are a prototype decision. Trait descriptions are prose,
not automatically executable mechanics or implication edges. Some otherTags have
family-specific finite vocabularies; they are not universally arbitrary strings.

Completion of the prototype requires stable output, visible unresolved constructs,
real shared references, compilable meaningful Rust output and focused fidelity
evidence. A compiling generator or corpus acceptance alone is insufficient.
Report handwritten code/policy size and maintenance exceptions before adoption.
No field-owner ledger, coverage receipts or new artifact readiness gate is needed.

## Related work

- [Localization-backed rules terms](./rust-localization-backed-rules-terms.md)
  owns later product lookup, hover content and record-to-term relationships.
- [Common Item parser PR16](https://github.com/e-kosten/pf2e-atlas/pull/16)
  and [shared physical parser PR17](https://github.com/e-kosten/pf2e-atlas/pull/17)
  are merged source-only components.
- [Physical-family refinements PR18](https://github.com/e-kosten/pf2e-atlas/pull/18)
  was open at the review above; authenticate its state before depending on it.

Canonical conversion, storage design, metrics, pipeline adoption, frontend source
inspection and broader family product work remain later milestones. This proposal
does not establish a production generator architecture.

Implementation lands through `integration/source-modeling`, starting from main.
The earlier `integration/record-refactor` branch is retained as research evidence
and a usable preview; no whole-crate or commit-chain import is planned. The
[selective recovery inventory](./rust-integration-recovery.md) records the useful
pieces, dependencies and deferred product work. Declaration graph extraction and
trait catalog extraction can be implemented independently above a shared tooling
base, with the combined command following both. gh-stack publication uses a linear
review chain, even when the implementation work happens in parallel.

## Extraction slice implemented

Offline tooling is under `scripts/source-contracts`; see its
[developer instructions](../../../scripts/source-contracts/README.md). The
reviewed PF2e pin produces 3,084 declaration graph nodes with 47 roots: five
pack document kinds covering all 24 Item and eight Actor families, plus 42
built-in rule schemas. Source discriminator sets match registered family sets.
The trait/tag/rarity output has 26 catalogs and 2,686 memberships. Explicit JSON
projections resolve the Predicate runtime array, ChoiceSet's constructor inputs,
and three omitted modifier callbacks while retaining their declaration provenance.
Graph and catalog extraction report complete; selected compiler diagnostics and
unsupported nodes are empty. The 226 unrelated upstream
project diagnostics and 32 localization warnings remain visible. Repeated
extraction is deterministic. Fixture tests cover shared identities, source
presence, portfolio additions, registry ambiguity and partial/strict command output.

This completes the bounded extraction and catalog slices. The
[equipment generation comparison](../../research/equipment-source-generation.md)
now generates equipped, hp, price and usage from 29 graph nodes, using one shared
physical/equipment value owner. Both the generated partial parser and compiled
PR18 manual reference accept 4,580 pinned occurrences; selected declared values
match throughout. Six legacy slot/deletion members are retained as ordered data
instead of separate typed properties. Eighteen adversarial comparison cases and
focused Rust/TypeScript tests exercise fidelity and fail-loud generation drift.
Saved inputs now use a manifest and physical/equipment snapshots with nodes owned
once; Rust output uses shared physical and equipment modules generated from the
whole selected graph. Freshness checks cover complete output sets. See
[ADR 0034](../../architecture/decisions/0034-source-generation-layout.md).
The production ingest pipeline remains unchanged.

The [shared Item slice](../../research/shared-item-source-generation.md) adds
description/publication, core trait fields for all 24 families and ordered keyed
item grants. Its 756-node input preserves declaration evidence, with 14 explicit
open trait-array policies. All 98,651 root/embedded occurrences across 22 families
match raw selected-value projections; fixtures cover absent book/affliction and
malformed selected values. Forbidden persisted fields remain additional data.
This does not establish full-family admission or full-portfolio generation.

A 47-root full generation attempt reports first blockers in recursion (34),
explicit upstream open domains (6), templates (5) and named-plus-indexed objects
(2). Next, implement recursion/richer unions and explicit open/indexed domain
semantics, then repeat generation across all roots. Nullable collection entries
and tuples also remain unsupported; first blockers are not an exhaustive list.
Full parser coverage and legacy semantic ownership still need separate evidence.
The focused predicate scan found one malformed upstream value combining `nor`
and `not` in revolutionary-innovation, rule 0, choice 29; Foundry's validator
also rejects it. Track this discrepancy during corpus comparison. The pinned
corpus has no nonempty custom modifiers, so their projections have fixture evidence.
Full-corpus agreement, frontend applicability, storage,
metrics and production pipeline adoption remain separate work.
