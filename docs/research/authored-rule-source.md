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
| Accepted | 20,037 | 31,157 |
| Rejected, still counted | 11,137 | 17 |
| Fidelity failures among accepted occurrences | 0 | 0 |

The projection recovers 11,120 occurrences, with zero acceptance regressions and
zero unmodeled rule keys. All occurrences of the six fully covered families
FlatModifier, Note, EphemeralEffect, Immunity, Resistance and Weakness, plus
RollTwice, AdjustModifier and DamageDice, now parse. ChoiceSet parses 1,527 of
1,528 occurrences; its remaining error is the confirmed malformed predicate.

Relative to PR 36's authored profile, 204 additional occurrences parse: 176
ChoiceSet, 23 DamageDice, four BattleForm and one Strike. Comparing the same packet
digest occurrence-by-occurrence shows zero regressions. Generic union fixes also
increase schema-profile acceptance by 63 before applying authored projections.

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

BattleForm parses 104 of 115 occurrences, and Strike parses 679 of 680. The
remaining failures stay counted below.

## Triage of all 17 remaining first errors

These classifications use the pinned implementation. They are research findings;
the generic comparison initially labels rejections unresolved and retains every
occurrence for review. Multiple defects in one rule can remain behind its first
error. Counts below sum to all 17 affected occurrences, not just unique examples.

| Group | Occurrences | Classification and evidence |
| --- | ---: | --- |
| ChoiceSet nested predicate with both nor/not | 1 | Confirmed upstream predicate error under the actual pinned StatementValidator; see below. |
| BattleForm strike baseType outside vocabulary | 10 | Unresolved declaration/corpus/runtime discrepancy; two cases were exposed after resolving nested IWR scalar inputs. No full Foundry Item validation run. |
| BattleForm strike range number | 1 | Unresolved declaration/corpus/runtime discrepancy: `BattleFormStrike.range` is an increment/max object, but the Nature Incarnate packet authors 100. The NumberField previously cited belongs to senses, not strikes. |
| Strike fist string | 1 | Unresolved coercion/legacy discrepancy; BooleanField does not establish acceptance without core execution. |
| TokenLight coloration numeric strings | 3 | Unresolved cleaning/choice-validation discrepancy; numeric coercion alone cannot establish valid coloration. |
| Sense bloodsense | 1 | Unresolved corpus/vocabulary discrepancy; the pinned Sense rule has finite StringField choices, and declarations alone do not establish migration/admission outcome. |

Totals: one confirmed upstream predicate error and 16 unresolved occurrences.
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

Resolve the 16 unexplained declaration/corpus/runtime discrepancies with targeted
upstream migration/runtime evidence. Actor/Item full-root emission still needs
explicit nullable/undefined collection modeling. This slice does not remove
those blockers or claim the entire source space is modeled.

Before adopting full-rule parsing into ingest, resolve unexplained rejections of
demonstrated valid inputs and unexplained value loss. Any upstream-error exception
needs specific implementation/runtime evidence and stays counted. Runtime
acceptance remains a separate evidence step, including migrations and concrete
parent context. Build ingest, database, metrics, product CLI, API and UI behavior
are unchanged by this work.
