# Complete source portfolio generation and document comparison

Date: 2026-10-07

## Source and scope

PF2e 6.12.4, authenticated pin `4cbdaa37d6c33e9519561bae2c59a23e0288cbce`,
TypeScript 5.9.3. The export reproduces source digest
`6bf64da835272af22c53db729cb28d18a87e8dee999c11b154fb3e772befdeb8`
over 1,509 files. Its complete graph has 3,085 nodes and 47 roots: five document
kinds, including all 24 Item/eight Actor families, plus 42 built-in rules.

All 47 roots now generate and compile individually and together against the
actual Rust primitives. Actor reaches the complete embedded Item union; common
shapes reuse owners across the portfolio. The combined generated portfolio stays
in scratch output. Production snapshot selections/generated modules, production
ingest, storage, metrics and UI remain unchanged. The committed generated additions
are synthetic regression fixtures, not a second maintained schema or corpus dump.

This completes generation for this pin. It establishes a measured document parser
baseline, not complete authored-document admission or production adoption.

## Generic capabilities

- Numeric indices retain canonical TypeScript number-property names as strings.
  Structs contain ordered typed indexed entries and ordered raw additional
  entries for other names. Named numeric keys satisfy field and index constraints;
  nonnumeric names do not acquire numeric-index constraints. Pure numeric maps
  anchor recursion. `BackgroundSystemSource.boosts` and numeric ItemUUID maps no
  longer block the portfolio. See [ADR 0037](../architecture/decisions/0037-open-and-indexed-source-values.md).
- Null-only fields use `SourcePresence<()>`; null/undefined-only unions have the
  same persisted null type. Missing/null remain facts before defaults; present
  non-null values fail. String enums support empty, numeric and punctuation tokens
  with valid Rust variant names and exact serde spellings. Naming collisions
  remain explicit errors. See [ADR 0035](../architecture/decisions/0035-source-value-generation-policy.md).
- A shared required, non-null field with pairwise disjoint finite literal domains
  identifies an object-union arm before defaults. The other fields retain normal
  presence parsing. Actor/Item `type` tags therefore work without `_stats`,
  `effects`, `ownership` and other commonly omitted fields. Overlapping or optional
  tags do not justify choosing an arm. Invalid present payloads still fail at
  their nested paths. See [ADR 0036](../architecture/decisions/0036-recursive-source-unions.md).

## Maintained comparison

`compare-documents` generates/compiles the entire extracted portfolio, then parses
every root Actor and root/embedded/nested Item from declared packs. It streams
exact JSON spans to a scratch Rust probe. JavaScript decodes context identifiers,
but never converts payload numbers and repeated fields into a new JSON document.
Rust compares accepted typed values with ordered raw source using graph shapes.
It checks values, presence, collection order and retained additional data.

All Actor/Item `system.traits.value` identifier arrays use the existing open-string
policy: 17 array identities across the complete portfolio. An explicit `never[]`
keeps its empty-array constraint. Other tags, rarity and small vocabularies retain
their declaration constraints. Open trait policies do not accept object-valued
trait entries or turn unrelated arrays into strings.

The report records source/graph/corpus identity, compiled roots, unobserved
registered families, per-family acceptance, fidelity failures, rejection groups
with examples, and every packet's first error. A failed repeat invalidates the old
report; a fully executed comparison has `status: "complete"` even when its
rejection count requires exit 1. Probe/build failures never produce an acceptance
claim. `compare-rules` uses the same compiler/probe runner with its existing
schema/authored profiles.

## Corpus evidence

Sampled document packet SHA-256:
`dde7c446213c82dcd22030b8909c623d18ce977c3a62841b4c4d409475a99451`.

| Document occurrences | Total | Accepted | Rejected |
| --- | ---: | ---: | ---: |
| Root Actors | 6,744 | 4,617 | 2,127 |
| Root/embedded/nested Items | 98,651 | 95,207 | 3,444 |
| Total | 105,395 | 99,824 | 5,571 |

All 99,824 accepted occurrences have zero measured fidelity failures. Six Actor
families and 22 Item families occur in the packs. Actor loot/party and Item
affliction/book have complete compiled declarations but no corpus occurrences.
Actor acceptance can fail because of an embedded Item; that Item is also sampled
separately. Counts are occurrences, not distinct bad records. First errors can
conceal later errors, and accepted additional/open data is preserved evidence
rather than a fully interpreted product field.

Seventeen observed families have no rejections. Remaining family counts:

| Family | Total | Rejected |
| --- | ---: | ---: |
| Actor/npc | 5,492 | 2,008 |
| Actor/character | 130 | 119 |
| Item/weapon | 4,676 | 2,480 |
| Item/spell | 26,265 | 882 |
| Item/ancestry | 178 | 28 |
| Item/consumable | 3,495 | 23 |
| Item/deity | 470 | 13 |
| Item/spellcastingEntry | 2,952 | 9 |
| Item/shield | 266 | 6 |
| Item/action | 28,419 | 2 |
| Item/armor | 1,546 | 1 |

The rule comparison remains 31,174 occurrences / 31,167 authored accepted, with
zero fidelity failures, regressions or unmodeled rule keys. Its counts and seven
remaining failure packets exactly match PR38. Their dispositions are unchanged.
See the [authored-rule report](./authored-rule-source.md).

## Remaining document boundaries

The report has 72 family/path/expected-type rejection groups. The following
partition covers all 5,571 first-error occurrences; it describes evidence and
next investigation, rather than adding parser exceptions.

| First-error pattern | Occurrences | Evidence / disposition |
| --- | ---: | --- |
| Empty weapon reload value | 3,859 | Upstream preparation handles this sentinel; declaration-only rejection does not establish bad data. |
| Empty weapon damage die | 147 | Upstream explicitly documents and handles this authored form. |
| Spell overlay heightening without a complete tag/payload | 1,071 | Overlays store diffs and are merged into a base spell. Standalone full-value union parsing is insufficient. |
| Number/boolean declarations receiving other JSON kinds | 335 | Includes numeric strings, empty strings and descriptive strings. Core cleaning/migration admission remains unexecuted. |
| Other incomplete heightening unions | 46 | Includes ordinary heightening objects lacking a type tag; inspect migration/source semantics separately from overlays. |
| Other vocabulary/template/structure disagreements | 113 | Includes old domains, PFS school, prepared-slot strings, invalid armor categories and object-valued traits. Keep exact contextual reports. |

Pinned upstream evidence:

- Weapon [`prepareBaseData`](https://github.com/foundryvtt/pf2e/blob/4cbdaa37d6c33e9519561bae2c59a23e0288cbce/src/module/item/weapon/document.ts#L312)
  applies `reload.value ||= null` and `damage.die ||= null`. It documents empty
  die strings used for constant damage. The Rust comparison preserves/reports
  these authored strings; it does not convert them into null.
- Spell [`updateOverride`](https://github.com/foundryvtt/pf2e/blob/4cbdaa37d6c33e9519561bae2c59a23e0288cbce/src/module/item/spell/overlay.ts#L53)
  saves a diff from the base spell. [`loadVariant`](https://github.com/foundryvtt/pf2e/blob/4cbdaa37d6c33e9519561bae2c59a23e0288cbce/src/module/item/spell/document.ts#L473)
  merges overlay system data into the source. [`DeepPartial`](https://github.com/foundryvtt/pf2e/blob/4cbdaa37d6c33e9519561bae2c59a23e0288cbce/types/foundry/util.d.ts#L7)
  does not recurse through an optional union containing undefined, explaining why
  the compiler retains the complete heightening union there. This is an authored
  patch boundary to model explicitly, not a reason to loosen every source union.

Before production adoption, define source-backed authored-document projections
for recurring sentinels and patch semantics, keeping original declarations and
raw values intact. Then rerun the full comparison to expose subsequent errors.
Keep isolated malformed/obsolete values diagnostic; do not add record allowlists,
speculative scalar coercions or a broad arbitrary-JSON fallback to make counts
green. Decide how production ingest preserves and reports rejected documents
before routing this stricter complete parser into it. No Foundry v12 runtime
cleaner/migration/admission test was executed.

## Validation and reproduction

The Rust fmt, both Clippy gates, workspace tests and build pass. TypeScript tests
cover generation freshness, bounded traversal, numeric key decisions against
TypeScript, index-domain sharing, null-only fields, exact enum token serialization,
tag-based union identity, trait policy boundaries and comparison failure reporting.
Compiled Rust fixtures cover numeric maps/recursion, ordered additional duplicates,
malformed values, fidelity corruption and union selection before defaults.

Use the [contributor command instructions](../../scripts/source-contracts/README.md#complete-document-comparison)
with the complete pinned source export and extraction. `compare-documents` returns
exit 1 for the reported rejections; `compare-rules` returns exit 1 for the seven
previously documented discrepancies. These are explicit measured outcomes.
Local outputs are under ignored `scratch/source-collections/document-comparison-fixed`
and `rule-comparison-expanded`, beside extraction and verification logs.
