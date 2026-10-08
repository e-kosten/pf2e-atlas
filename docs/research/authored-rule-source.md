# Authored rule inputs and corpus fidelity

## Source and boundary

PF2e 6.12.4 at `4cbdaa37d6c33e9519561bae2c59a23e0288cbce`, TypeScript
5.9.3. Source input digest:
`6bf64da835272af22c53db729cb28d18a87e8dee999c11b154fb3e772befdeb8`
(1,509 files). Packs were exported from the same pinned revision. Comparison
reports also identify the exact sampled rule packets by SHA-256.
The sampled packet digest for this run is
`f539f02f13b0802bcc0adbbd5a76d670c1386e1f341c1b3b6eba3ecb12cb4ab3`.

The schema graph remains complete for 47 roots. Companion schema aliases expose
field classes without running `defineSchema`. Node/value shapes in the existing
production snapshots and generated modules are unchanged. Rule-root array field
metadata is discovery provenance, separate from their extracted value nodes.

The ordinary ArrayField/StrictArrayField distinction comes from pinned upstream
implementation, not from guessing a union from corpus values. PF2e's
[StrictArrayField](https://github.com/foundryvtt/pf2e/blob/4cbdaa37d6c33e9519561bae2c59a23e0288cbce/src/module/system/schema-data-fields.ts#L111)
overrides casting and cleaning specifically to avoid ordinary scalar wrapping.
Foundry documents
[cleaning and validation as distinct DataField operations](https://foundryvtt.com/api/v12/classes/foundry.data.fields.DataField.html).
No local Foundry core cleaner/DataModel bootstrap was executed. The evidence
supports modeling these authored forms; it does not demonstrate complete runtime
admission or rule execution with a real Actor/Item parent.

## Modeled inputs

Ten selected collection fields have a separate authored projection:

- AdjustModifier.selectors, DamageDice.selector, EphemeralEffect.selectors,
  FlatModifier.selector, Note.selector and RollTwice.selector.
- Immunity.type, Resistance.type and Weakness.type.
- Strike.traits, with the same explicit open/closed vocabulary policy for the
  scalar and array alternatives.

Only ordinary ArrayField of strings is projected. DamageAlteration.selectors and
IWR exceptions/doubleVs are strict arrays and retain that boundary. Other array
fields retain their extracted shape. The IWR projection also reaches BattleForm's
nested immunity/resistance/weakness objects through their shared compiler
declaration origin and original value reference. These objects are passed into
the corresponding IWR rule constructors by
[`#prepareIWR`](https://github.com/foundryvtt/pf2e/blob/4cbdaa37d6c33e9519561bae2c59a23e0288cbce/src/module/rules/rule-element/battle-form/rule-element.ts#L450).
Unrelated `type` arrays do not change; schema nodes and declaration/serialization
provenance remain intact. The comparison reports the three nested derived owners
as `sharedIwrChanges`, separately from the ten root field policies.

For example, both `"selector":"attack"` and `"selector":["attack","damage"]`
become typed union variants. Serialization retains their different authored
forms; parsing does not turn the scalar into an array. IWR types remain strings,
including injected-property text, because this schema's element is StringField;
dictionary validation occurs later in IWR rule preparation. Missing/null states
remain SourcePresence states, rather than synthesized defaults or admission claims.

ChoiceSet config, owned-item and attack-query predicates may be omitted in authored
objects. The [constructor](https://github.com/foundryvtt/pf2e/blob/4cbdaa37d6c33e9519561bae2c59a23e0288cbce/src/module/rules/rule-element/choice-set/rule-element.ts#L67)
supplies `predicate ?? []`; the three interfaces describe that populated object.
The projection marks the predicate optional for union identity and preserves its
missing/null/value state without supplying a default. Pack-query `filter` stays
required. Config/owned-item/attack forbidden keys retain their declaration origin.

DamageDice override `damageType` and `dieSize` accept authored strings and
`diceNumber` accepts numbers or expression strings. Its
[`#isValidOverride`](https://github.com/foundryvtt/pf2e/blob/4cbdaa37d6c33e9519561bae2c59a23e0288cbce/src/module/rules/rule-element/damage-dice.ts#L155)
explicitly accepts these string forms; preparation resolves them before checking
vocabularies and numeric constraints. Prepared upgrade/downgrade booleans retain
their extracted types. `valueChanges` identifies the six changed declaration
fields and their provenance. This represents source expressions, not evaluation
or proof that every expression resolves successfully.

BattleForm's nested strike `baseType` also has an authored string projection,
bringing `valueChanges` to seven fields. Its [schema](https://github.com/foundryvtt/pf2e/blob/4cbdaa37d6c33e9519561bae2c59a23e0288cbce/src/module/rules/rule-element/battle-form/rule-element.ts#L106)
stores `strikes` as ObjectField, and the [preparation handoff](https://github.com/foundryvtt/pf2e/blob/4cbdaa37d6c33e9519561bae2c59a23e0288cbce/src/module/rules/rule-element/battle-form/rule-element.ts#L384)
assigns `baseItem: strikeData.baseType`. It does not assign Strike's `baseType`
field, whose StringField has the closed base-weapon choices. Those choices cannot
establish a restriction on this authored member. Preserve the string, null and
missing states without changing direct Strike or the shared weapon vocabulary.
This models the observed source boundary; it does not repair the upstream naming
discrepancy or establish the intended gameplay effect.

Generic union selection uses declaration-forbidden key absence as well as
required keys/literals. Anchored shapes take precedence over broad open and
optional-only fallbacks when their value kinds overlap. Competing anchors or
competing fallbacks remain errors. Malformed anchored payloads retain their
nested error rather than being accepted as open JSON. This distinguishes ChoiceSet
forms and bracketed RuleValue objects without handwritten Rust rule dispatch.
See [ADR 0036](../architecture/decisions/0036-recursive-source-unions.md).

## Reproduction

Use a complete pinned source export, including `packs`, `static/system.json`,
`src`, `types`, compiler configuration and dependencies. From the Atlas root:

```sh
npm --prefix scripts/source-contracts ci --ignore-scripts
npm --prefix scripts/source-contracts run extract -- \
  --source scratch/pf2e --out scratch/authored-rules/extraction --strict
npm --prefix scripts/source-contracts run compare-rules -- \
  --source scratch/pf2e \
  --graph scratch/authored-rules/extraction/type-graph.json \
  --summary scratch/authored-rules/extraction/summary.json \
  --policy-manifest scripts/source-contracts/snapshots/manifest.json \
  --out scratch/authored-rules/comparison
```

The manifest supplies the existing explicit open-trait-array policy, verified
against the same source digest. Omit it to compare the closed extracted
vocabularies instead, retaining any trait-vocabulary rejections. The command
requires a local Cargo toolchain and dependencies already
cached for its offline scratch builds. It does not install or invoke Foundry.

Output: `comparison.json`, per-occurrence `schema.json` and `authored.json`, two
generated scratch portfolios and an isolated Cargo target directory. The report
retains contextual first errors for every rejected occurrence and separately
counts unmodeled keys. It exits 1 on this pin because outstanding discrepancies
remain, including the known malformed predicate. There is no success allowlist.

## Results

All 42 built-in rule roots emitted and compiled together in each profile. Forty
keys occur in packs; TokenImage and TokenName have no corpus occurrences.

| Check | Schema shapes | Authored projection |
| --- | ---: | ---: |
| Rule occurrences sampled | 31,174 | 31,174 |
| Accepted | 20,037 | 31,167 |
| Rejected, still counted | 11,137 | 7 |
| Fidelity failures among accepted occurrences | 0 | 0 |

The projection recovers 11,130 occurrences, with zero acceptance regressions and
zero unmodeled rule keys. All occurrences of the six fully covered families
FlatModifier, Note, EphemeralEffect, Immunity, Resistance and Weakness, plus
RollTwice, AdjustModifier and DamageDice, now parse. ChoiceSet parses 1,527 of
1,528 occurrences; its remaining error is the confirmed malformed predicate.

Relative to PR 36's authored profile, 204 additional occurrences parse: 176
ChoiceSet, 23 DamageDice, four BattleForm and one Strike. Comparing the same packet
digest occurrence-by-occurrence shows zero regressions. Generic union fixes also
increase schema-profile acceptance by 63 before applying authored projections.
Relative to PR 37, the nested BattleForm base-type policy recovers ten more
occurrences. The packet digest is unchanged, with zero regressions or fidelity
failures. No further first errors are exposed in those recovered occurrences.

Fidelity comparison runs in Rust before results pass through JavaScript. It
compares every graph-declared field and presence state, recursively represented
values and ordered additional/typed-map members with original SourceValue. This
preserves integers beyond JavaScript's safe range and duplicate additional keys.
It is independent of the emitter's generated field list and parsing decisions;
it does share the discovered graph and public serialization convention, so it
cannot establish that discovery itself found every meaningful upstream field.
Corruption fixtures demonstrate detection of omitted modeled fields, null/missing
collapse, scalar-to-array coercion, large-integer changes, lost duplicates and
reordered additional/map entries. Generated fixtures exercise strict arrays,
scalar/array variants, empty arrays and contextual rejection.

BattleForm parses 114 of 115 occurrences, and Strike parses 679 of 680. The
remaining failures stay counted below.

## Triage of all seven remaining first errors

These classifications use the pinned implementation. They are research findings;
the generic comparison initially labels rejections unresolved and retains every
occurrence for review. Multiple defects in one rule can remain behind its first
error. Counts below sum to all seven affected occurrences, not just unique examples.

| Group | Occurrences | Evidence-backed disposition | Remaining evidence |
| --- | ---: | --- | --- |
| ChoiceSet nested predicate with both nor/not | 1 | Confirmed invalid predicate under the actual pinned StatementValidator. Keep rejection. | Full document admission unexecuted; predicate validity is established. |
| BattleForm strike range number | 1 | Legacy-shaped input: the handoff retains 100 and Migration868 updates only direct Strike rules. Keep the increment/max object model. | Foundry SchemaField cleaning and the complete migration chain. |
| Strike fist string | 1 | Wrong declared field type and suspicious authoring: the initializer treats truthy values as fixed-fist mode, never as a claw name. Keep boolean representation. | Actual BooleanField cleaning/admission of this string. |
| TokenLight coloration numeric strings | 3 | Wrong declared field type and suspected technique/intensity confusion. The pinned editor selects numeric technique IDs and provides a separate alpha control. Keep numeric representation. | Actual LightData cleaning, technique choices and shader behavior. |
| Sense bloodsense | 1 | Unsupported upstream vocabulary: both the declaration and the executed implementation's allowed-choice set omit this sense. Keep the closed vocabulary; this is not a missing compiler enum member. | Full Sense/DataModel admission unexecuted. |

Totals: one confirmed invalid predicate, one established upstream vocabulary
conflict and five occurrences with unresolved core/migration behavior. The generic
comparison continues to label all rejections unresolved; this research supplies
their dispositions without changing acceptance or introducing an allowlist.
None are ignored or declared valid merely because they occur in packs. None are
declared invalid merely because our
generated parser rejects them.

The numeric BattleForm strike range was previously classified as a known
authored-model gap based on the wrong field. The pinned BattleForm schema leaves
`strikes` as an ObjectField and passes each strike range into Strike unchanged.
The upstream [range migration](https://github.com/foundryvtt/pf2e/blob/4cbdaa37d6c33e9519561bae2c59a23e0288cbce/src/module/migration/migrations/868-strike-re-range.ts#L9)
converts direct Strike rules, not nested BattleForm strikes. This evidence does
not establish migration or runtime acceptance of that nested numeric range, so
the parser remains constrained and this occurrence stays unresolved.

Focused execution of the pinned BattleForm `ruleData` mapping expression on all
11 previously rejected BattleForm occurrences confirms that every nested
`baseType` reaches `baseItem`, no `baseType` property is emitted, and `range`
passes through unchanged. The mapping was selected with the TypeScript AST and
executed without rewriting its field assignments. Localization/icon dependencies
were supplied only to run the mapping; no Foundry cleaner, rule constructor,
Actor/Item admission or gameplay behavior was simulated.

The actual pinned Migration868 `updateItem` method was also transpiled and
executed with a direct Strike and nested BattleForm range of 100. It changed
the direct Strike to `{ increment: 100 }` and left the nested range at 100.
This is evidence about that migration only, not a full migration-chain run.

The pinned `SENSE_TYPES` initializer was executed and does not contain
`bloodsense`. Sense's `defineSchema` uses exactly that set as StringField choices,
so this discrepancy is present in the implementation as well as the declaration;
it is not evidence of a missing compiler enum member. Full document admission
remains unexecuted. Strike's `fist` remains BooleanField, and TokenLight forwards
the three coloration strings to LightData without an explicit PF2e coercion.
Foundry's core cleaner/validator is unavailable locally; neither accepting these
values nor declaring their runtime rejection is justified by the current probes.

## Detailed dispositions and targeted controls

The investigation uses the same source and packet digests as the comparison.
It does not edit packs, synthesize defaults or repair the corpus. The source
manifest declares Foundry compatibility 12.328 through v12, verified at 12.331.
No matching Foundry runtime is available, so core-dependent claims stay unverified.
Foundry's documented [DataField cleaning](https://foundryvtt.com/api/v12/classes/foundry.data.fields.DataField.html#clean)
and validation are distinct operations; an API cast signature alone cannot
establish what a particular value becomes or whether it passes later validation.

### Unsupported sense vocabulary

`packs/feats/general/bloodsense.json`, `$.system.rules[0].selector`, authors
`"bloodsense"`. Its description intentionally grants that sense, so this is not
a justified inference of a spelling mistake. The [Sense schema](https://github.com/foundryvtt/pf2e/blob/4cbdaa37d6c33e9519561bae2c59a23e0288cbce/src/module/rules/rule-element/sense.ts#L22)
requires a non-null selector with choices from the exact `SENSE_TYPES` set.
The executed set omits `bloodsense`. The [rule base](https://github.com/foundryvtt/pf2e/blob/4cbdaa37d6c33e9519561bae2c59a23e0288cbce/src/module/rules/rule-element/base.ts#L46)
disables fallback during DataModel construction. This establishes an upstream
feature/vocabulary conflict, with no evidence for widening Atlas's source enum.
A `scent` substitution is a parser control only, not an equivalent feature or
a proposed repair. The actual feat remains rejected and counted.

### Legacy nested strike range

`packs/spell-effects/spell-effect-nature-incarnate.json`,
`$.system.rules[1].overrides.strikes["thorns"].range`, authors 100. The executed
handoff leaves it numeric, and the actual Migration868 method leaves that nested
member unchanged while converting a direct Strike range to `{ increment: 100 }`.
The nested shape therefore lacks demonstrated migration support. Retain the
current object representation and the contextual rejection; do not add a numeric
union merely because this resembles the legacy range representation. The object
control parses, but it does not establish whether increment or maximum range
matches the intended spell. Complete migration/core behavior remains a gap.

### String in fixed-fist mode

`packs/triumph-of-the-tusk-bestiary/book-1-the-resurrection-flood/stormblood-tiger.json`,
`$.items[5].system.rules[2].fist`, authors `"claw"`. The [Strike initializer](https://github.com/foundryvtt/pf2e/blob/4cbdaa37d6c33e9519561bae2c59a23e0288cbce/src/module/rules/rule-element/strike.ts#L145)
contains `if (this.fist)`, which supplies a fixed fist's identity, traits and damage.
Executing that exact branch with `"claw"` or `true` sets `baseType` to `fist`
and replaces the authored slashing die/modifier with one d4 bludgeoning die and
modifier zero. With `false`, it leaves the authored damage intact. This isolates
the initializer branch; it does not execute BooleanField cleaning or prove that
the string reaches that branch in a real DataModel. There is no special meaning
for a claw-valued fist flag. Keep boolean parsing and the rejection. Both boolean
controls parse, but choosing a repair would require authoring intent and runtime
evidence; the source description itself says an orc fist, not a claw.

### Light technique versus intensity

The three sources are `crownbound-constellation.json`,
`$.items[5].system.rules[0].value.coloration`, and `mindmoppet.json`,
`$.items[15].system.rules[0].value.coloration` and
`$.items[15].system.rules[1].value.coloration`, all under
`packs/gatewalkers-bestiary/book-3-dreamers-of-the-nameless-spires/`.
They author `"0.45"`, `"0.4"` and `"0.4"` respectively.

The pinned [TokenLight editor](https://github.com/foundryvtt/pf2e/blob/4cbdaa37d6c33e9519561bae2c59a23e0288cbce/src/module/item/base/sheet/rule-element-form/token-light.ts#L16)
obtains choices from `AdaptiveLightingShader.SHADER_TECHNIQUES`. Its
[template](https://github.com/foundryvtt/pf2e/blob/4cbdaa37d6c33e9519561bae2c59a23e0288cbce/static/templates/items/rules/token-light.hbs#L94)
renders `value.coloration` as a Number-typed select using technique IDs.
The same template has a distinct `value.alpha` range control from zero to one.
The fractions resemble intensity values, which is an authoring-error hypothesis,
not a demonstrated repair. The actual v12 technique IDs and LightData validator
are absent from the source export. Keep numeric parsing and retain all three
rejections; do not map coloration to alpha or change their values automatically.

Both technique number 1 and a numeric version of each fraction parse as shape
controls. This also exposes a boundary: the generated Number parser checks
representation, not shader-technique membership. Parser acceptance of a fractional
number does not establish semantic validity. A future runtime probe must inspect
the cleaned value, validation failures and the actual selected technique.

### Probe results

Focused probes used the same compiled authored portfolio as the full comparison:
seven original packets all reject at the reported field, and twelve controls all
parse with zero fidelity errors. Controls comprise two isolated predicate
operators, one known sense, one range object, both fist booleans, and two numeric
forms for each of the three lights. They are test inputs, not corpus fixes.
The initializer branch, vocabulary initializer and individual migration probes
execute selected pinned upstream code; none simulates a Foundry core cleaner.
The template was retrieved from the exact PF2e commit; it is UI evidence outside
the 1,509-file compiler-input digest. No new production parser policy follows
from these remaining cases.

For the one confirmed predicate error, the exact input is
`packs/classfeatures/revolutionary-innovation.json`,
`$.system.rules[0].choices[29].predicate[2]`:

```json
{"nor":["feature:blunt-shot","weapon-innovation:thrown"],"not":"feature:blunt-shot"}
```

We transpiled and invoked the actual pinned `src/module/system/predication.ts`
with real Remeda, without a mocked validator. `StatementValidator.isStatement`
returned false, and `Predicate.isValid` returned false for the containing
predicate. Each isolated nor-only and not-only statement returned true. The
upstream compound validators require one operator key. This establishes the
predicate's structural invalidity; complete document/rule admission was not run.

## Follow-up and adoption boundary

The agreed source-model disposition is to retain the current constrained shapes
and report these seven occurrences as input problems. No case-specific parser,
coercion, field widening, pack edit or repair is warranted by this investigation.
This is an intentional modeling decision; it does not reclassify the five
core/migration-dependent occurrences as proven invalid at runtime. A future
source update or actual runtime evidence may warrant revisiting the policy.

All seven remaining occurrences now have a recorded source-model disposition.
Five occurrences still require core/migration evidence, and complete document
admission is unexecuted throughout. These are explicit runtime evidence gaps,
not resolved admission claims. The unsupported sense needs upstream vocabulary
support; Atlas does not broaden that enum independently. Keep confirmed invalid
data rejected and visible. Actor/Item full-root emission still needs
explicit nullable/undefined collection modeling. This slice does not remove
those blockers or claim the entire source space is modeled.

Before adopting full-rule parsing into ingest, resolve unexplained rejections of
demonstrated valid inputs and unexplained value loss. Any upstream-error exception
needs specific implementation/runtime evidence and stays counted. Runtime
acceptance remains a separate evidence step, including migrations and concrete
parent context. Build ingest, database, metrics, product CLI, API and UI behavior
are unchanged by this work.
