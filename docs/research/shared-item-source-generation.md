# Shared Item source generation

Date: 2026-10-06

## Selected model

Compiler-derived generation now covers shared Item description, publication,
core traits (value/rarity/otherTags) and keyed item grants across all 24 registered
Item families. The earlier physical/equipment fields remain available. This is
source-only: production ingest, canonical conversion, storage, metrics and UI
do not call or adopt these parsers.

The callable `parse_item_source_slice` returns typed selected components and
ordered additional system/envelope/flag members. Its trait enum dispatches by
registered family, while aliases share actual value owners. Common description,
publication and grant values live in `generated/items/common.rs`; trait models
and dispatch live in `generated/items/traits.rs`. Existing physical/equipment
modules retain their owners. Support includes typed vectors and ordered
`SourceMap<T>` entries, with contextual errors for malformed selected values or
repeated modeled keys.

Trait toggles, shield integrated traits and spell traditions are explicitly
deferred. They remain ordered additional values. Legacy declaration-forbidden
members also remain additional data: effect rarity and class/lore empty value
arrays occur in the corpus. Missing required members are ordinary pre-default
source facts, not replaced with Foundry defaults.

Fourteen exact trait-array identities have an explicit open-string generation
policy. Original declaration vocabulary nodes remain saved. Current finite
other-tag vocabularies, rarity, publication license and grant deletion behavior
remain checked sets. Treasure's declared `never[]` remains empty-only. No
array-size threshold or global string-enum widening is used.

Deity's traits union contains OtherTagsOnly plus three compiler-proven impossible
intersections. Extraction retains their members and records `impossible: true`
using compiler assignability to `never`. Selection uses the sole viable arm;
`sourceRef` retains the original union and closure in saved input. Empty objects
alone are not discarded.

## Inputs and size

- Base: integration at `7558e07af1b713ece871a3b20bd92a946483ca82`.
- PF2e 6.12.4: `4cbdaa37d6c33e9519561bae2c59a23e0288cbce`, exported from the
  vendor Git object rather than its different working revision.
- TypeScript 5.9.3, 1,509 input files, source digest
  `6bf64da835272af22c53db729cb28d18a87e8dee999c11b154fb3e772befdeb8`.
- Dependency lock digest
  `7a529fe3429ec19e948efe519f54b1ba2a2da8b45cfb07a0e8c1fd06da7b4747`.

Saved input has 29 roots and 756 nodes owned once across common Item, trait,
physical and equipment snapshots. Most of the larger trait snapshot retains
declaration/vocabulary evidence even when Rust uses open strings. It is static
JSON input, not validation machinery. Output is 1,111 generated Rust lines across
four content modules and two indexes. Handwritten Rust remains in source
primitives and slice composition; tooling extends the existing generator.

Roots are grouped by module, shared owners before refinements. Interleaved roots
reject rather than changing snapshot ownership on reload. Complete-file freshness
checks cover metadata, policies and output. A policy target changing to a nonstring
array fails generation. Unsupported selected values have no arbitrary-JSON
fallback. See [ADR 0035](../architecture/decisions/0035-source-value-generation-policy.md).

## Corpus and fixture evidence

Recursive AST-span sampling found **98,651 Item source occurrences** in declared
Item/Actor packs: 18,634 root and 80,017 embedded/nested Items across 22 families.
The Rust probe accepted all 98,651 and matched every selected-value projection,
including missing/null states and ordered additional members: **zero rejections
or value differences**. Sampling includes nested Items; typing recursive item
containment remains future work.

The oracle reads ordered JSON in Rust and projects selected fields using snapshot
metadata. It does not copy the generated parser or decide source admission.
Numbers remain in Rust until comparison transport. Additional members retain
order and repetitions. This proves selected-slice fidelity; a shared selection
mistake could escape comparison, so declaration coverage needs separate review.

Book and affliction have no observed corpus records. Rust fixtures exercise their
nonempty common/trait values and family-specific additional-member behavior.
Other fixtures cover all 24 dispatches, missing/null/empty states, future traits,
closed-set failures, empty-only traits, nullable grant members, ordered maps,
duplicate rejection, large integers and nested diagnostics. TypeScript fixtures
protect graph drift, representation policy, impossible intersections, sharing,
indexed-intersection rejection, nullable collection-owner deduplication, sampler
fidelity and complete artifact freshness.

## Whole-portfolio attempt and next work

A dry run attempted each of the extraction's 47 full roots without writing
output. None yet generates completely. The first blocker per root groups as:

| First blocker | Roots |
| --- | ---: |
| Recursion, primarily PredicateStatement | 34 |
| Explicit upstream open object domains | 6 |
| Template literal values | 5 |
| Named-plus-indexed objects | 2 |

This reports first blockers, not every missing construct behind them. Richer
unions, nullable collection entries, tuples and recursive embedded documents also
remain unsupported. The next capability slice should handle recursion/unions and
explicitly declared open/indexed domains, then repeat the full-root attempt.
Exactly one more PR is not established.

## Reproduce

Follow the [source package instructions](../../scripts/source-contracts/README.md)
to export the pin and run complete strict extraction. Then:

```sh
npm --prefix scripts/source-contracts run verify
cargo test -p atlas-ingest --test item_source_generation --test equipment_source_generation
cargo build -p atlas-ingest --example item_generation_probe
set -o pipefail
node scripts/source-contracts/dist/sample-items.js --source scratch/pf2e | \
  target/debug/examples/item_generation_probe > scratch/item-corpus-report.json
```

Repeat the first-blocker attempt after extraction:

```sh
node --input-type=module <<'JS'
import fs from 'node:fs';
import { generateRustModules } from './scripts/source-contracts/dist/source-generation.js';
import { loadGenerationInput } from './scripts/source-contracts/dist/generation-input.js';
const graph = JSON.parse(fs.readFileSync('scratch/source-extraction/type-graph.json'));
const summary = JSON.parse(fs.readFileSync('scratch/source-extraction/summary.json'));
const input = await loadGenerationInput('scripts/source-contracts/snapshots/manifest.json');
const results = graph.roots.map(root => {
  try {
    generateRustModules({ source: summary.source, nodes: graph.nodes,
      openTraitArrays: input.openTraitArrays,
      selection: [{ name: 'PortfolioProbe', declaration: root.ref, module: 'probe',
        deferred: [], fields: [{ name: 'value', ref: root.ref, optional: false,
          nullable: false, undefinedAllowed: false, forbidden: false, declaredAt: [] }] }] });
    return { root: root.ruleKey ?? root.name, generated: true };
  } catch (error) { return { root: root.ruleKey ?? root.name, generated: false, firstBlocker: error.message }; }
});
console.log(JSON.stringify(results, null, 2));
JS
```
