# Authored document inputs and corpus fidelity

Date: 2026-10-07

## Source and measured boundary

PF2e 6.12.4 at `4cbdaa37d6c33e9519561bae2c59a23e0288cbce`, TypeScript
5.9.3. Source digest:
`6bf64da835272af22c53db729cb28d18a87e8dee999c11b154fb3e772befdeb8`
over 1,509 files. Sampled packet SHA-256:
`dde7c446213c82dcd22030b8909c623d18ce977c3a62841b4c4d409475a99451`.
The extracted graph remains unchanged: 3,085 nodes and 47 roots, including all
24 Item/eight Actor families and 42 built-in rule schemas.

`compare-documents` measures declaration and authored profiles on the same raw
packets. Both complete portfolios generate and compile against actual Rust source
primitives. The authored graph preserves the original nodes and adds scoped
source representations; it changes no maintained Rust selections, production
ingest, storage, metrics or UI. See
[ADR 0040](../architecture/decisions/0040-authored-document-inputs.md).

## Source-backed representations

Weapon [`prepareBaseData`](https://github.com/foundryvtt/pf2e/blob/4cbdaa37d6c33e9519561bae2c59a23e0288cbce/src/module/item/weapon/document.ts#L312)
handles empty reload/die strings, explicitly documenting the die sentinel used
for constant damage. It also defaults empty bonus/splash damage values to zero.
The projection retains empty strings beside the existing vocabulary or numeric
type. It does not convert them into null/zero, accept arbitrary numeric strings
for weapon damage values or widen other owners of those vocabularies.

Spell [`prepareBaseData`](https://github.com/foundryvtt/pf2e/blob/4cbdaa37d6c33e9519561bae2c59a23e0288cbce/src/module/item/spell/document.ts#L600)
uses area-value truthiness and numeric division and supplies a missing area shape.
Area values therefore retain number/string forms, including empty or unresolved
text; area shapes admit the empty sentinel beside their vocabulary. This models
the input representation, not a validated distance. Later normalization must
interpret and report unsuitable text. Other numeric/boolean fields retain their
declaration types.

Spell [`_preUpdate`](https://github.com/foundryvtt/pf2e/blob/4cbdaa37d6c33e9519561bae2c59a23e0288cbce/src/module/item/spell/document.ts#L1164)
clears empty damage categories in base, overlay and fixed heightening updates.
It also clears passive/save defenses when their statistic is empty. Those
sentinels remain strings in the authored model; the Rust parser does not apply
these update operations or assert document admission.

[`ActorInitiative`](https://github.com/foundryvtt/pf2e/blob/4cbdaa37d6c33e9519561bae2c59a23e0288cbce/src/module/actor/initiative.ts#L30)
accepts a statistic slug and resolves it through
[`getStatistic`](https://github.com/foundryvtt/pf2e/blob/4cbdaa37d6c33e9519561bae2c59a23e0288cbce/src/module/actor/base.ts#L401),
including lore and synthetic statistics. Its authored statistic uses a string;
the declaration's fixed skill vocabulary does not cover that lookup surface.
Parsing a slug does not establish that a particular Actor has that statistic.

Spell [`updateOverride`](https://github.com/foundryvtt/pf2e/blob/4cbdaa37d6c33e9519561bae2c59a23e0288cbce/src/module/item/spell/overlay.ts#L53)
saves an object diff. [`loadVariant`](https://github.com/foundryvtt/pf2e/blob/4cbdaa37d6c33e9519561bae2c59a23e0288cbce/src/module/item/spell/document.ts#L470)
merges both override and fixed heightening layer systems into the base spell.
Both contexts share a generic recursive object-patch representation. Nested
objects/maps may omit members, arrays/tuples retain replacement element shapes,
and partial object unions combine field domains without claiming to select a
complete union arm. A heightening patch such as `{"damage":{"x":"1d6"}}` can
omit the base's type tag. An ordinary full heightening value still needs a tag
identifying the fixed/interval arm. Validity of the merged spell is a separate
contextual check.

The compiler's [`DeepPartial`](https://github.com/foundryvtt/pf2e/blob/4cbdaa37d6c33e9519561bae2c59a23e0288cbce/types/foundry/util.d.ts#L7)
does not recurse through an optional union containing undefined. Its complete
heightening union therefore disagrees with saved diffs. The projection models
the explicit merge boundary rather than changing extraction or loosening full
unions globally. All policies record declaration origins and source evidence;
shape drift fails visibly. Updating the source pin also requires checking the
implementations, which can change without declaration changes.

## Corpus result

| Profile | Occurrences | Accepted | Rejected | Fidelity failures |
| --- | ---: | ---: | ---: | ---: |
| Declaration baseline | 105,395 | 99,824 | 5,571 | 0 |
| Authored documents | 105,395 | 105,113 | 282 | 0 |

The authored profile recovers 5,289 occurrences with zero acceptance regressions.
Of 6,744 root Actors, 6,569 accept and 175 reject. Of 98,651 root/embedded/nested
Items, 98,544 accept and 107 reject. All accepted occurrences have zero measured
fidelity failures. The 12 document field policies reuse generic generation and
patch support; they introduce no handwritten Rust family parser.

Every rejected occurrence remains in 37 family/path/type groups and in the
full contextual failure list. No packet is repaired, suppressed or allowlisted.

| Remaining first-error pattern | Occurrences |
| --- | ---: |
| PFS school `"none"` | 73 |
| Number fields receiving strings | 75 |
| Boolean `applyMod` receiving strings | 20 |
| Full heightening `{"damage":{}}` without a type tag | 48 |
| Vocabulary conflicts (movement, damage, deity domains, armor, NPC adjustment) | 46 |
| Collection/element structure conflicts (slots, runes, traits) | 14 |
| Legacy compendium UUID spelling | 4 |
| Prepared `validItems` UI label | 2 |
| Total | 282 |

Actor rejection can be caused by an embedded Item which is also sampled
separately. Counts describe occurrences, not distinct faulty records. Every
packet reports its first error; correcting a supported form can expose a later
problem. The same four families remain unobserved: Actor loot/party and Item
affliction/book. Their declaration portfolios still compile.

Item's upstream `rules` member is a generic `RuleElementSource` array, not a
discriminated union of the 42 schemas. Document acceptance preserves those
payloads and does not certify each specific rule. The separate rule comparison
remains 31,174 occurrences / 31,167 authored accepted, zero fidelity failures or
regressions and the same seven failure packets, counts and packet digest as
PR38/PR39. Their [dispositions](./authored-rule-source.md) are unchanged.

## Remaining constrained inputs

The modeling decision for these patterns is to retain their declared forms.
Runtime admission remains unexecuted; that gap does not establish that every
rejected document is bad. These decisions avoid reproducing broad core coercion
or adding compatibility forms solely to accept isolated old values, following
[ADR 0039](../architecture/decisions/0039-authored-rule-inputs.md).

- Number/boolean fields receive strings: ancestry feature levels, focus maxima,
  prepared-slot maxima, HP, ritual caster counts and spell `applyMod`. Retain
  numbers/booleans. Strings such as `"0"` are particularly unsafe to interpret
  as booleans through JavaScript truthiness. The available export does not
  supply Foundry's core cleaner or `Math.clamp` implementation. Spell-area
  representation has the separate direct arithmetic evidence above.
- Full heightening is exactly `{"damage":{}}`, outside a patch context. Upstream
  preparation/getHeightenLayers check `type === "fixed"` or `"interval"` and do
  not interpret this as either form. Keep the full union constrained rather than
  invent a third heightening mode or silently supply its missing tag.
- PFS school is `"none"`; current declarations use three schools or null.
  Other closed vocabulary conflicts include old deity domains, movement labels,
  damage types such as `"healing"`, armor `"simple"`, and empty NPC adjustment.
  Retain current vocabularies. Migration
  [895](https://github.com/foundryvtt/pf2e/blob/4cbdaa37d6c33e9519561bae2c59a23e0288cbce/src/module/migration/migrations/895-fix-variant-spell-traits.ts#L78)
  explicitly names legacy healing damage and translates base damage to untyped;
  it does not prove admission of the nested heightening packets in this report.
- Prepared slots contain names rather than objects, property runes contain an
  object rather than an array, or trait arrays contain selector objects rather
  than strings. Keep collection/element shapes. Spellcasting preparation
  [reads slot object members](https://github.com/foundryvtt/pf2e/blob/4cbdaa37d6c33e9519561bae2c59a23e0288cbce/src/module/item/spellcasting-entry/document.ts#L100),
  and open trait identifiers remain strings; no name-to-slot or widget-object
  compatibility conversion is introduced.
- Old compendium references use `.db.` where the declared Item UUID template
  requires an Item document segment. Keep the template constraint and raw
  diagnostic, without a legacy UUID rewrite.
- Prepared `validItems` contains `"All Magic Items"` rather than the current
  scroll/empty/null forms. Upstream's
  [fallback treats non-scroll values alike](https://github.com/foundryvtt/pf2e/blob/4cbdaa37d6c33e9519561bae2c59a23e0288cbce/src/module/item/spellcasting-entry/document.ts#L236).
  That fallback does not justify modeling arbitrary old UI labels as a new
  supported vocabulary. Keep the declaration boundary and admission uncertainty.

These are value/representation decisions, rather than missing generic graph
syntax. Preserve raw rejected input and contextual diagnostics. Before pipeline
adoption, decide the rejection policy, specific-rule dispatch and contextual
patch validation/normalization. Foundry cleaning, migration execution and complete
document admission still have no runtime evidence.

## Validation and reproduction

The Rust workspace fmt, broad/strict Clippy, tests and build pass. Node 22
typechecking and fixture tests include actual Rust probes for recursive patches,
unions without tags, maps, complete replacement arrays/tuples, sentinels, custom
statistic slugs, invalid present values, additional duplicates and numeric
fidelity. Existing generation freshness checks still pass without changes to
maintained source snapshots or Rust output. Workflow lint passes; CI fetches
locked Rust dependencies before the offline fixture probes.

Run [compare-documents](../../dev-tools/source-contracts/README.md#complete-document-comparison)
and `compare-rules` with the authenticated source export and matching complete
extraction. Both return exit 1 for the explicitly retained rejections, after
writing complete reports. Their status means the comparison executed, not that
every input was admitted. Diagnostic evidence is retained under ignored
`scratch/authored-document-generation` in the shared checkout.
