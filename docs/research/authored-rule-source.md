# Authored rule inputs: selectors and IWR

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

Nine selected fields now have a separate authored projection:

- AdjustModifier.selectors, DamageDice.selector, EphemeralEffect.selectors,
  FlatModifier.selector, Note.selector and RollTwice.selector.
- Immunity.type, Resistance.type and Weakness.type.

Only ordinary ArrayField of strings is projected. DamageAlteration.selectors and
IWR exceptions/doubleVs are strict arrays and retain that boundary. Other array
fields and nested BattleForm IWR inputs remain separate follow-up work.

For example, both `"selector":"attack"` and `"selector":["attack","damage"]`
become typed union variants. Serialization retains their different authored
forms; parsing does not turn the scalar into an array. IWR types remain strings,
including injected-property text, because this schema's element is StringField;
dictionary validation occurs later in IWR rule preparation. Missing/null states
remain SourcePresence states, rather than synthesized defaults or admission claims.

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
vocabularies instead; that deliberately adds 46 trait-vocabulary rejections on
this pin. The command requires a local Cargo toolchain and dependencies already
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
| Accepted | 19,974 | 30,900 |
| Rejected, still counted | 11,200 | 274 |
| Fidelity failures among accepted occurrences | 0 | 0 |

The projection recovers 10,926 occurrences, with zero acceptance regressions and
zero unmodeled rule keys. All occurrences of the six fully covered families
FlatModifier, Note, EphemeralEffect, Immunity, Resistance and Weakness, plus
RollTwice and AdjustModifier, now parse. DamageDice's remaining 23 first errors
concern override fields, not selectors.

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

## Triage of all 274 remaining first errors

These classifications use the pinned implementation. They are research findings;
the generic comparison initially labels rejections unresolved and retains every
occurrence for review. Multiple defects in one rule can remain behind its first
error. Counts below sum to all 274 affected occurrences, not just unique examples.

| Group | Occurrences | Classification and evidence |
| --- | ---: | --- |
| ChoiceSet choices with missing predicate | 116 | Authored-model gap: ChoiceSet's constructor supplies `predicate ?? []`; its interfaces describe the populated object while union identity currently requires that field. |
| ChoiceSet choices with overlapping arms | 60 | Parser defect: optional attack-query fields create a competing arm for config/owned-item inputs. Runtime `inflateChoices` dispatches on config/ownedItems/attacks; the generator's generic union identity does not reproduce that distinction. |
| ChoiceSet nested predicate with both nor/not | 1 | Confirmed upstream predicate error under the actual pinned StatementValidator; see below. |
| DamageDice override.damageType injection strings | 16 | Authored-model gap: `beforePrepareData` resolves injected properties before dictionary validation. |
| DamageDice override.diceNumber formulas | 7 | Authored-model gap: `#isValidOverride` accepts strings and preparation resolves the expression before numeric validation. |
| BattleForm nested IWR scalar type | 55 | Authored-model gap: `#prepareIWR` passes these objects into ordinary IWR rule constructors. This PR's root-only projection does not yet apply there. |
| BattleForm strike baseType outside vocabulary | 8 | Unresolved declaration/corpus/runtime discrepancy; no full Foundry Item validation run. |
| BattleForm strike range number | 1 | Authored-model gap: the declared BattleForm schema and its actual NumberField describe numeric range, whereas the embedded prepared strike interface describes an object. |
| BattleForm bracketed resistance value overlapping open/object arms | 4 | Parser defect: broad/open and structured RuleValue alternatives compete in generic union selection. |
| Strike scalar traits | 1 | Authored-model gap: ordinary ArrayField outside this PR's bounded target fields. |
| Strike fist string | 1 | Unresolved coercion/legacy discrepancy; BooleanField does not establish acceptance without core execution. |
| TokenLight coloration numeric strings | 3 | Unresolved cleaning/choice-validation discrepancy; numeric coercion alone cannot establish valid coloration. |
| Sense bloodsense | 1 | Unresolved corpus/vocabulary discrepancy; the pinned Sense rule has finite StringField choices, and declarations alone do not establish migration/admission outcome. |

Totals: 196 authored-model gaps, 64 parser defects, one confirmed upstream
predicate error and 13 unresolved occurrences. None are ignored or declared valid
merely because they occur in packs. None are declared invalid merely because our
generated parser rejects them.

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

Prioritize ChoiceSet authored identity and expression-valued DamageDice overrides;
then carry shared authored IWR semantics into nested BattleForm objects and resolve
the remaining prepared/source discrepancies. Actor/Item full-root emission still
needs explicit nullable/undefined collection modeling. This slice does not remove
those blockers or claim the entire source space is modeled.

Before adopting full-rule parsing into ingest, resolve unexplained rejections of
demonstrated valid inputs and unexplained value loss. Any upstream-error exception
needs specific implementation/runtime evidence and stays counted. Runtime
acceptance remains a separate evidence step, including migrations and concrete
parent context. Build ingest, database, metrics, product CLI, API and UI behavior
are unchanged by this work.
