# Compiler-based source contracts and trait catalog

Status: in_progress
Priority: next source-modeling experiment
Owner: unassigned
Last reviewed: 2026-10-08

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
Extraction and complete pinned Rust portfolio generation are implemented.
Production adoption and full Foundry admission remain separate milestones.

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

The [authored-rule comparison](../../research/authored-rule-source.md) implements
authored/cleaned input separation for selectors, nested IWR types, ChoiceSet
predicates, DamageDice override expressions, scalar Strike traits and nested
BattleForm strike base-type strings. Generic
union selection distinguishes anchored shapes from open/optional-only fallbacks.
It recovers 11,130 of 31,174 rule occurrences without measured value loss;
Seven first-error occurrences remain counted and have documented dispositions:
one confirmed malformed predicate, one unsupported upstream sense and five
core/migration-dependent cases. Keep the current field representations and
report these input problems; do not add case-specific parsing or repairs.
Full Foundry admission remains unexecuted, with explicit evidence gaps rather
than claims that every remaining value is runtime-invalid. These dispositions
permit continued generator work. Persisted nullable/undefined collections are
supported. All 47 roots now generate and compile together, including Actor/Item.
Full-document corpus parsing has a reproducible comparison command; authored
empty sentinels, contextual spell diffs and migration/core-dependent discrepancies
remain explicit adoption work. See the [document report](../../research/document-source-generation.md).
Before production adoption, define how
ingest preserves and reports rejected source rules, and verify demonstrated
valid authored forms without introducing record allowlists.

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

Offline tooling is under `dev-tools/source-contracts`; see its
[developer instructions](../../../dev-tools/source-contracts/README.md). The
reviewed PF2e pin produces 3,085 declaration graph nodes with 47 roots: five
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
Rust output uses shared physical and equipment modules generated from the
whole selected graph. Declaration evidence is regenerated from the source pin
into the ignored cache. Freshness checks cover complete output sets. See
[ADR 0034](../../architecture/decisions/0034-source-generation-layout.md).
The production ingest pipeline remains unchanged.

The [shared Item slice](../../research/shared-item-source-generation.md) adds
description/publication, core trait fields for all 24 families and ordered keyed
item grants. Its 756-node input preserves declaration evidence, with 14 explicit
open trait-array policies. All 98,651 root/embedded occurrences across 22 families
match raw selected-value projections; fixtures cover absent book/affliction and
malformed selected values. Forbidden persisted fields remain additional data.
This does not establish full-family admission or full-portfolio generation.

The [recursive capability slice](../../research/recursive-source-generation.md)
adds anchored recursion, mixed unions and fixed tuples, using complete persisted
predicates as the concrete model. All 18,512 accepted selected corpus values
match raw projections; one known upstream nor/not conflict remains reported.
Compiled generic fixtures exercise states absent from the corpus. The complete
source graph is unchanged; original serialization provenance closes generation inputs.

The [intersection capability slice](../../research/intersection-source-generation.md)
brings the 47-root attempt to 45 emitted and individually compiled roots: all 42
rule roots plus JournalEntry, Macro and RollTable. Open/indexed domains, string
templates, concrete generic naming and field-name mappings now have explicit
representations. Intersection index constraints come directly from the compiler.
The [collection capability slice](../../research/collection-source-generation.md)
adds inline Option entries for persisted nullable arrays, fixed tuples and maps,
plus null-only entries. Undefined array positions persist as null; undefined
object properties remain absent. Shape signatures intern shared descendants and
validate recursive anchors without unbounded expansion. That capability slice
left the then-maintained generated modules unchanged. The
[document capability slice](../../research/document-source-generation.md)
adds numeric indices, null-only fields, unusual literal enum tokens and
discriminator-only selection when required finite literal domains are disjoint.
All 47 roots compile together. `compare-documents` samples all root Actors and
root/embedded/nested Items, checks raw-to-model fidelity, applies open trait
identifiers across all families and reports first-error groups and absent corpus
families. Whole-portfolio generated output stays in scratch; no corpus value is
repaired or allowlisted. The
[authored-document projection](../../research/authored-document-source.md) models
upstream-supported sentinels, spell area number/string forms, custom initiative
statistic slugs and recursive patches for overrides/fixed heightening layers.
The original declaration graph remains available beside authored results,
recovery/regression counts and fidelity diagnostics. The complete authored
portfolio is maintained under `crates/atlas-ingest/src/source_model/generated`,
with a pinned source-to-Rust workflow and callable document/specific-rule parsers.
The contributor `compare-portfolio` command tests these maintained models against
all document kinds and specific rules, preserving every first-error context and
checking value fidelity. A pinned corpus baseline detects changed input,
rejection sets, counts and value loss; known rejections remain nonzero diagnostic
outcomes. Remaining migration/core-dependent or malformed values
retain explicit constrained dispositions. Callable source admission now preserves
useful documents with raw invalid fields and contextual diagnostics, using
`SourcePresence::Invalid`; specific rejected rules remain wholly unavailable to
typed interpretation. NPC senses use compiler-resolved constructor inputs before
defaults. See [ADR 0041](../../architecture/decisions/0041-source-admission-and-field-retention.md).
Production normalization/storage adoption and contextual normalization remain open.
Typed source loading is available through `load_foundry_documents` and the private
Rust `atlas-dev source load` command. It retains generated document DTOs, original
bytes/hashes, raw values, embedded source relationships, provenance and all
admission diagnostics. Malformed envelopes have explicit quarantine outcomes;
unavailable packs remain visible. No current product normalization or artifact
writing is adopted by this stage. See
[ADR 0042](../../architecture/decisions/0042-typed-source-loading.md).
The next checkpoint is a
[combined normalization and database review](./typed-ingest-database-design.md),
including ground-up scrutiny of metrics and measured FTS, semantic and filter
query patterns before storage commitments.
Before choosing
field/node recovery or whole-document rejection, report affected unique document
names, families and source paths, retained useful content, and losses under each
policy. Counts of overlapping rejected occurrences alone do not justify a drop
policy; a small number of marginal records may be acceptable, while useful or
commonly needed records merit narrower recovery. Do not call
all reported rejections bad data or treat
generation completion as complete authored admission.
Optional/rest tuples and alias-only recursive collections remain unsupported
general capabilities; they do not block the pinned portfolio. Emission/compilation does not establish
accepted or value-faithful family coverage. The authored rule corpus exposes
scalar/array, vocabulary and union-identity discrepancies; compare authored source
interfaces and Foundry cleaning with SourceFromSchema before adopting those parsers.
Full parser coverage and legacy semantic ownership still need separate evidence.
The focused predicate scan found one malformed upstream value combining `nor`
and `not` in revolutionary-innovation, rule 0, choice 29; Foundry's validator
also rejects it. Track this discrepancy during corpus comparison. The pinned
corpus has no nonempty custom modifiers, so their projections have fixture evidence.
Full-corpus agreement, frontend applicability, storage,
metrics and production pipeline adoption remain separate work.
