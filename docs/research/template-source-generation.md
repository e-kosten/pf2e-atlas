# Template domains, generic names and portfolio compilation

Date: 2026-10-07

## Implemented capability

The emitter supports template strings with arbitrary string interpolations and
concrete generic-instantiation names. The pinned complete graph contains 39
template domains, all with string interpolations. Examples include `#${string}`,
`${string}.png`, `Actor.${string}.Item.${string}` and compendium UUID patterns.
Generated String aliases retain parser constraints, including empty, multiline
and Unicode interpolations. File/color templates do not imply additional runtime
validation. Overlapping string patterns share one string domain; mixed union,
tuple/array guards and required object discriminants retain pattern constraints.
Other interpolation domains remain explicit errors.

Named generic arguments yield names such as
SourceFromSchemaChoiceSetRuleSchema. Anonymous/complex arguments use field or
root context. Equal resolved shapes continue to share one owner, and genuine
residual collisions remain errors. There are no numeric naming counters or Rust
generic schema implementations. See [ADR 0038](../architecture/decisions/0038-source-templates-and-generic-names.md).

Committed production snapshots and generated models remain unchanged: 791
selected nodes, six snapshots and 1,754 production Rust lines across nine files.
The full-root outputs below are diagnostic artifacts, not adopted family models.
Production ingest, database, metrics, app contracts and UI remain deferred.

## Input identity and verification

- Integration base: bdcf3d5f14f300fcd9a9b99b4f57ff61d6ae3c69, merged PR33.
- PF2e 6.12.4: 4cbdaa37d6c33e9519561bae2c59a23e0288cbce.
- TypeScript 5.9.3; complete saved extraction has 3,084 nodes and 47 roots,
  source digest 6bf64da835272af22c53db729cb28d18a87e8dee999c11b154fb3e772befdeb8.
  Extraction code is unchanged; this task reuses that pinned extraction rather
  than claiming a new extraction run.
- Packs and static/system.json are exported from the same vendor Git object;
  the vendor working revision is not the source input.
- Package verification passes 62 tests on Node 22.23.3; the full Rust gate passes.
- The generated regression fixture compiles against the actual Rust primitives
  and runs 20 tests. New cases cover templates, overlapping patterns, required
  template discriminants, tuple/mixed guards and shared generic owners.
- Graph-mode freshness checks cover the unchanged production output and snapshots.
- The existing independent Item fidelity probe accepts and matches all 98,651
  root/embedded occurrences, with zero value differences. This validates the
  existing selected slice, rather than full rule-family fidelity.

## Whole-portfolio compilation

Per-root emission improves from 27 to **41 of 47 roots**. All 41 outputs compile
individually in an isolated Cargo harness against the actual source presence,
ordered value/map, parse and union primitives. Compilation does not establish
corpus admission or full-value fidelity. No whole-portfolio combined ownership
or pipeline integration claim is made.

| Remaining first blocker | Roots |
| --- | --- |
| Indexed intersections | ActorSourcePF2e, ItemSourcePF2e |
| Rust field name `_id` | JournalEntrySource, MacroSource, RollTableSource |
| Rust field name `greater-darkvision` | BattleForm |

First blockers are not an exhaustive inventory. Actor/Item flags occur in
intersections, beyond the named-plus-indexed object support already implemented.
Field naming needs a deliberate escaped/mapped representation before these
roots can emit. Fixing these may reveal further shapes.

## Authored rule corpus

The sampler walks the same root/recursively embedded Items as the existing Item
probe, taking raw AST spans of each direct `system.rules` entry. It preserves
numeric tokens and repeated members for Rust parsing. There are **31,174** rule
occurrences across 40 keys in this pin. The blocked BattleForm root accounts for
115 unsampled parser attempts. TokenImage and TokenName have no occurrences.

The 41 compiled rule parsers attempt the remaining **31,059** occurrences:
**19,927 parse and serialize**, while **11,132 reject** across 12 rule families.
The probe exits 1 and retains every source context/diagnostic; there is no allowlist.
This measures parser acceptance and serialization success. It does not compare
the complete serialized rule model to an independent raw-value projection.

| Family | Rejected | First diagnostic categories |
| --- | ---: | --- |
| ChoiceSet | 177 | 176 missing/ambiguous choices alternatives; one known malformed predicate |
| DamageDice | 2,083 | 2,060 scalar selectors; 16 override damage types; seven override dice numbers |
| EphemeralEffect | 12 | Scalar selectors |
| FlatModifier | 6,187 | Scalar selectors |
| Immunity | 144 | Scalar type |
| Note | 1,470 | Scalar selectors |
| Resistance | 817 | Scalar type |
| RollTwice | 58 | Scalar selectors |
| Sense | 1 | `bloodsense` outside the declared vocabulary |
| Strike | 2 | Scalar traits; string-valued fist |
| TokenLight | 3 | String-valued coloration |
| Weakness | 178 | Scalar type |

These are first diagnostics per occurrence, not exhaustive bad-field counts or
a conclusion that all rejected sources are invalid to Foundry. For example,
FlatModifier's constructor accepts FlatModifierSource, whose selector is optional
JSONValue, while defineSchema/SourceFromSchema resolves selector to string[].
DamageDice similarly accepts a source interface before its resolved schema.
Several damage override values also pass through injected-property resolution.
The existing source declarations and runtime cleaning/interpretation therefore
need comparison before choosing an authored-source model. ChoiceSet failures can
reflect both source forms and the current strict object-union identity policy.
The revolutionary-innovation predicate remains the known upstream conflict.

Do not silently widen these declarations or normalize scalars into arrays in
source-preserving parsers. The next modeling work must distinguish authored input,
cleaned source and runtime values explicitly, while keeping this evidence.

## Next work

1. Finish field-name mapping and indexed-intersection representations; repeat
   emission and compilation rather than assuming those are the last gaps.
2. Compare authored rule interfaces and Foundry cleaning/coercion with
   SourceFromSchema. Model accepted source forms deliberately, preserving the
   authored value and contextual failures. Investigate ChoiceSet union identity
   separately from actual source errors.
3. Run whole-root fidelity/admission checks before adopting generated families.
   Continue deferring database/product interpretation and metric changes.

## Reproduce

Use the [source tooling instructions](../../dev-tools/source-contracts/README.md)
to export the pin and extract it completely. The per-root generation loop in the
[recursive report](./recursive-source-generation.md#reproduce) works with this
emitter. For each emitted root, write its files under a separate generated
directory in an isolated Cargo package with serde/serde_json and edition 2024.
Define a source_model module loading the actual keyed, parse, presence, union and
value modules through path attributes, re-export SourceMap and load the generated
mod.rs. Run cargo check for every root. Keeping each output separate avoids
making a combined ownership claim.

For authored rules, sample Items using sampleItems, locate direct system.rules
entries with TypeScript's JSON AST, and pass raw spans plus source context to the
corresponding compiled parse_portfolio_probe. Keep rejected diagnostics and return
nonzero when any occur. This is a contributor experiment, not a product CLI
command or release dependency.

Durable local artifacts are under ignored scratch/template-source-generation:
portfolio.json, compilation.json, rule-counts.json, rule-corpus.json and per-root
compile.log/corpus.json. The temporary scripts record the original harness.
The standard Rust/TypeScript gates cover committed generated fixtures; the
full-portfolio corpus remains an explicitly failing diagnostic experiment.
