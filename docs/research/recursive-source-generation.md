# Recursive source generation and predicate comparison

Date: 2026-10-07

This report records PR32. The subsequent
[open/indexed source report](./open-source-generation.md) records the current
flags model and the updated 27-root emission result.

## Implemented capability

The existing emitter supports anchored recursive models, mixed unions and fixed
tuples. Its concrete selected model is complete persisted PredicateStatement
(string plus 13 operator objects), Predicate arrays, and ChoiceSet's existing
statement-or-array constructor-input projection. No predicate execution,
production ingest, storage, metrics or UI adoption occurs.

The Rust API exposes `parse_predicate_statement`, `parse_predicate_statements`
and `parse_predicate_input` with contextual diagnostics. Generated values live
under `source_model::generated`; owners are partitioned into `rules/predicate.rs`.
Comparison operands share a typed `(String, StringOrNumber)` tuple and parser;
the tuple appears inline in all five operator fields. Anonymous primitive unions
use member-derived names in fixed String, Number, Boolean order. Complete
true/false pairs represent Boolean; restricted literal alternatives retain their
checks. Declared/selected root names can retain aliases, and name collisions fail
explicitly. These generated source types are inputs to later ingest interpretation,
with application, storage and predicate execution models deferred. Array
operators share one vector owner. Negation and conditional inline cycles use
boxed union payloads; arrays/maps already provide indirection. Remaining inline
cycles are checked after removing boxed union edges.

Union identity uses value kind, required keys and literal discriminants. Tuple
alternatives also use arity and positional scalar constraints; empty `never[]`
matches only empty arrays. Exactly one candidate must match. Competing keys are
ambiguous even when one payload is null or malformed. A selected arm checks
required member uniqueness/nullability and retains nested parser errors.
Complex overlapping alternatives can still be ambiguous; emission alone does
not establish that every declared value is accepted.

Ordinary objects retain SourcePresence before defaults and ordered additional
members. An incomplete competing key set, such as `then` beside a `not` statement,
remains additional data. This models TypeScript source shapes without imposing
all Foundry runtime constraints: empty atomic strings/arrays and unknown added
members can be retained. Runtime admission and predicate evaluation are separate.
See [ADR 0036](../architecture/decisions/0036-recursive-source-unions.md).

## Candidate and saved input

- Integration base: `11978a40255cdefacb005f79cc471dfaeb72077e` (PR31 merged).
- PF2e 6.12.4: `4cbdaa37d6c33e9519561bae2c59a23e0288cbce`, exported from the
  vendor Git object; no dependency on its different working revision.
- TypeScript 5.9.3; 1,509 source inputs; source digest
  `6bf64da835272af22c53db729cb28d18a87e8dee999c11b154fb3e772befdeb8`.
- Fresh strict extraction remains complete: 3,084 nodes and 47 roots.

Selected input contains 781 nodes owned once across five module snapshots,
including a new 750-line predicate snapshot. Original Predicate array `rawRef`
and ChoiceSet field `declaredRef` provenance close over saved nodes. The original
PickableThing declaration remains evidence without making its other fields
selected parser values. Generated source Rust totals 1,627 lines across eight
files, with 500 in the predicate module. The trait snapshot remains unchanged.

Recursive nominal owners retain upstream identity; other value shapes share
structural owners. Union selection facts stay distinct even when ordinary
pre-default payload structs can share. Parser ownership stays separate from
primitive Rust type imports and public root aliases.

## Corpus and fixture evidence

The selected field sampler discovers 18,513 source occurrences:

| Field/context | Occurrences |
| --- | ---: |
| predicate | 17,560 |
| definition, including nested exceptions | 511 |
| ChoiceSet choices.filter | 343 |
| CraftingAbility craftableItems | 31 |
| RollOption disabledIf | 63 |
| Predicate-valued removeAfterRoll | 5 |

ChoiceSet `choices[i].predicate` uses constructor input (1,257 occurrences,
including three singleton statements). Other selected contexts retain array-only
inputs; malformed nonarrays are sampled as discrepancies. Ten ItemAlteration
definitions are labeled legacy discovery, not evidence of a declared family
field owner. This is bounded field/value discovery, not a whole-source schema
coverage inventory.

The Rust parser accepts **18,512 occurrences**, all equal to raw typed/presence/
additional-member projections. There are **zero value differences** and one
reported upstream rejection:

`packs/classfeatures/revolutionary-innovation.json`,
`$.system.rules[0].choices[29].predicate[2]` contains both `nor` and `not`.
The generated parser reports `JointDenial | Negation` ambiguity. Foundry's pinned
runtime validator also rejects this operator combination. The probe exits 1;
there is no parser or probe allowlist. The discrepancy is not an extraction error.

Both predicate and existing Item probes compare JSON values directly in Rust,
retaining exact integers and number/string distinctions. Their raw value
projections are not independent admission validators; selection/coverage still
needs separate review. The stricter Item comparison remains equal for all 98,651
root/embedded occurrences.

Fixtures cover all 14 alternatives, recursive conditionals/negations, comparison
arity/positions, missing/null/duplicate identity members, unknown/additional
members and nested diagnostics. Test-only synthetic graphs compile against the
actual source primitives, covering mixed object/union cycles, recursive tuple
cycles, empty/single tuples, nullable literals, raw keyword fields, primitive
literal roots/imports and optional-arm ambiguity. A DivineFonts-shaped union
exercises empty, harm, heal and harm/heal alternatives by arity and literals.
Package tests check the complete saved fixture output, not only string fragments.
Probe tests detect partial operator-key projection errors and number/string loss.
Scalar-union fixtures cover all supported primitive combinations, member-order
stability, cross-module sharing/imports and literal restrictions. Tuple fixtures
also compile when an anonymous tuple root is selected before its recursive union.

## Whole-portfolio attempt

The refreshed attempt emits seven of the 47 complete roots: ActorTraits,
AdjustDegreeOfSuccess, RollTwice, SpecialStatistic, TokenEffectIcon, TokenMark and
TokenName. This diagnostic calls the emitter; these full-root outputs are not
adopted models and have no portfolio compilation/admission claim.

| First remaining blocker | Roots |
| --- | ---: |
| Explicit upstream open object domains | 31 |
| Template literal domains | 6 |
| Named-plus-indexed objects | 2 |
| Generic-instantiation Rust name collision | 1 |

Next handle explicit open/indexed domains, template domains and deterministic
generic naming, then repeat the attempt. Nullable collection entries, optional/
rest tuples and alias-only recursive types remain unsupported. First blockers
do not inventory every construct behind them or establish how many PRs remain.

## Reproduce

Follow the [package instructions](../../dev-tools/source-contracts/README.md) to
export the pin and extract it strictly. From the repository root:

```sh
npm --prefix dev-tools/source-contracts run verify
cargo test -p atlas-ingest --test predicate_source_generation \
  --test recursive_generation_fixture --example predicate_generation_probe
cargo build -p atlas-ingest --example predicate_generation_probe
set -o pipefail
node dev-tools/source-contracts/dist/src/cli/sample-predicates.js --source scratch/pf2e | \
  target/debug/examples/predicate_generation_probe > scratch/predicate-corpus-report.json
```

Inspect the one expected upstream rejection rather than treating the nonzero
corpus-probe exit as a passing test. No compatibility fallback is applied.

Repeat the complete-root emission attempt:

```sh
node --input-type=module <<'JS'
import fs from 'node:fs';
import {generateRustModules} from './dev-tools/source-contracts/dist/src/generation/source-generation.js';
import {loadGenerationInput} from './dev-tools/source-contracts/dist/src/generation/generation-input.js';
const graph=JSON.parse(fs.readFileSync('scratch/source-extraction/type-graph.json'));
const summary=JSON.parse(fs.readFileSync('scratch/source-extraction/summary.json'));
const input=await loadGenerationInput('dev-tools/source-contracts/snapshots/manifest.json');
const results=graph.roots.map(root=>{
  try {
    generateRustModules({source:summary.source,nodes:graph.nodes,
      openTraitArrays:input.openTraitArrays,
      selection:[{name:'PortfolioProbe',declaration:root.ref,valueRef:root.ref,
        module:'probe',fields:[],deferred:[]}]});
    return {name:root.ruleKey??root.name,emitted:true};
  }catch(error){return {name:root.ruleKey??root.name,emitted:false,firstBlocker:error.message};}
});
console.log(JSON.stringify(results,null,2));
JS
```
