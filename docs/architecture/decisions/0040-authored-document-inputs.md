# Authored document forms and object patches

Status: accepted
Date: 2026-10-07

## Decision

Keep extracted declaration graphs unchanged. The private TypeScript generator
derives a separate authored document graph from bounded, source-backed policies,
composed with the existing authored rule roots. Generated Rust preserves input
values and presence before preparation; it does not execute Foundry cleaning,
migrations, patch application or admission validation.

The document policies verify declaration owners, field origins and expected
shapes. Weapon reload/damage-die and spell damage-category enums admit an empty
string where pinned upstream preparation/update code handles that sentinel.
Weapon bonus/splash damage values admit the same empty sentinel beside numbers;
spell passive/save defense statistics admit it beside their vocabularies.
Other owners sharing those vocabularies retain their constraints. Spell area
values admit numbers or strings, and area shapes admit the empty sentinel:
upstream preparation uses truthiness, numeric division and a default shape.
String values remain strings, including unresolved text; numeric interpretation
and quality diagnostics belong to later normalization. This does not generalize
number/string handling to other numeric fields or boolean fields.

Creature initiative statistics use string slugs. Upstream `ActorInitiative`
resolves them through `actor.getStatistic`, which includes lore and synthetic
statistics beyond the declaration's fixed skill list. This field's parser
preserves the slug without proving that a given Actor has that statistic.

Spell override systems and fixed heightening layer systems are object patches.
`updateOverride` stores diffs, while `loadVariant` merges these systems into a
base spell before constructing the variant. Both use one generic patch-shape
projection:

- Named object members become optional; nested object and map values recurse.
- Arrays and tuples remain replacement values with their original element and
  position constraints. Their elements do not become object patches.
- Object union alternatives combine into one partial object containing the
  alternatives' fields and their value domains. A saved diff can omit a tag or
  contain only a shared member, so it cannot identify a complete union arm yet.
  Complete source unions elsewhere retain their existing tag/shape constraints.
- Supplied scalar values, enum members, null states, map key domains and
  additional data retain their parsing and fidelity rules. Recursive patches
  reuse references. Missing nodes, incompatible index domains and impossible
  intersections fail visibly rather than becoming arbitrary JSON.

A patch's field-domain validity does not establish validity of the merged spell.
Later contextual application must check the resulting complete value. The
projection supplies neither defaults nor merge behavior. Copied ancestor nodes
carry policies through shared declarations and cycles; original declaration and
serialization provenance remains intact.

When updating the upstream pin, recheck preparation, update and variant-merge
implementations as well as declaration shapes. Identity and shape checks alone
cannot detect semantic changes that leave those declarations unchanged.

`compare-documents` compiles both complete portfolios against real Rust source
primitives and parses the same raw packets. It reports schema/authored counts,
recovered occurrences, acceptance regressions and fidelity failures, retaining
every rejected context and grouped first error. Rejections, regressions or
fidelity failures return exit 1. A completed diagnostic run can have rejections;
failed commands invalidate their old report.

Item documents retain upstream's generic `RuleElementSource` array. Document
acceptance is therefore not a claim that every specific rule was parsed. The
separate rule comparison exercises the specific rule models and their authored
policies. Corpus overlap between Actors and their separately sampled Items also
means rejection counts are occurrences, not counts of distinct bad records.

## Consequences

Full generated portfolios remain diagnostic scratch output. Maintained Rust
source selections, production ingest, database models, metrics and UI remain
unchanged. No per-record repairs, runtime cleaner replica or receipt gate is
introduced. Remaining discrepancies retain declaration constraints and explicit
runtime evidence gaps under [ADR 0039](./0039-authored-rule-inputs.md).

The [document authored-input report](../../research/authored-document-source.md)
records implementation evidence and the remaining decisions. Before pipeline
adoption, decide how rejected raw documents are preserved/reported and how
contextual patches and specific rules are parsed and normalized.
