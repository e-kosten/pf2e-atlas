# Bounded equipment source generation

Date: 2026-10-06

This report records the bounded PR30 trial. The current selection and emitter
have since expanded through the [shared Item slice](./shared-item-source-generation.md);
the counts, supported constructs and recommendation below describe PR30.

## Result and recommendation

Compiler-derived Rust works for the selected physical/equipment fields without
empty structs or discarded selected values. Continue with a bounded shared Item
component before committing to whole-portfolio generation. Keep the production
ingest pipeline separate until parser coverage and conversion policy are settled.

This experiment types only `system.equipped`, `hp`, `price` and `usage`. Equipment
and physical declarations share one Rust value owner for these fields. The full
equipment closure contains 1,476 nodes through recursive subitems and other
physical families; the selected generation input contains 29 nodes.

## Candidates and inputs

- Generated candidate: `feat/equipment-source-generation`, based on source-modeling
  integration at `ad8b242db8809ca5f0cccabd4b13cef908f502e4`.
- Compiled manual reference: PR18 head
  `1ef0ab8fd13abf02e809a8ff47dd0ee9c3f97b29`. The validation oracle imports that
  checkout's actual `parse_physical_item_source`; it contains serialization of
  four typed results, not a copied parser.
- PF2e 6.12.4: `4cbdaa37d6c33e9519561bae2c59a23e0288cbce`, exported from its Git
  object. The current vendor checkout was a different revision and was not used.
- Complete extraction: TypeScript 5.9.3, 1,509 input files, source digest
  `6bf64da835272af22c53db729cb28d18a87e8dee999c11b154fb3e772befdeb8`, dependency lock
  digest `7a529fe3429ec19e948efe519f54b1ba2a2da8b45cfb07a0e8c1fd06da7b4747`.

PR30's [input manifest](https://github.com/e-kosten/pf2e-atlas/blob/99aed39d8e3af5f54f5276b44a9b05ce60cc2156/dev-tools/source-contracts/snapshots/manifest.json)
records source identity and points to physical/equipment snapshots. Together they
retain selected declarations, source locations, optional/null/undefined facts and
deferred field names. Each of the 29 graph nodes occurs once. This is a compact
graph selection, not a coverage ledger. The [PR30 Rust modules](https://github.com/e-kosten/pf2e-atlas/blob/99aed39d8e3af5f54f5276b44a9b05ce60cc2156/crates/atlas-ingest/src/source_model/generated/mod.rs)
are readable and reproducible without an upstream checkout. Equipment imports
shared physical owners. See [ADR 0034](../architecture/decisions/0034-source-generation-layout.md)
for partitioning and regeneration rules.

## Comparison evidence

Manifest-driven sampling covered 2,178 root equipment Items and 2,402 equipment
entries directly under `Actor.items`: **4,580 occurrences**. Recursive subitems
were retained as pending data, not counted as additional modeled occurrences.
Both parsers accepted all 4,580. Comparisons inspect typed field values, presence
states and ordered additional members; they do not compare only acceptance or
raw JSON round trips.

| Result | Occurrences |
| --- | ---: |
| Equal complete comparison projections | 4,574 |
| Equal declared values, different legacy-field ownership | 6 |
| Other value or admission differences | 0 |

Five embedded Items have an empty `equipped.slot`; one has a null `-=inSlot`
deletion marker. The manual reference types these legacy fields separately.
Generation retains them in ordered `additional_fields`. Each of the six complete
differences was inspected: removing only that legacy member from the generated
comparison projection yields exact equality. No field was dropped by the parser.

All 4,580 have typed hp, price and usage objects. Equipped is missing on 2,178
roots and present on 2,402 embedded Items. The corpus contains no nulls at these
four object roots, so adversarial fixtures supply that coverage.

The checked comparison fixture has 18 cases: five equal typed results, ten equal
rejections with matching paths, and three deliberate scope differences. A legacy
slot/deletion case is preserved as additional data. Invalid legacy slot and
unknown size are accepted by this partial parser but rejected by the broader
manual parser. This is not equivalent full-family admission.

Rust tests separately assert missing/null/empty states, every selected value
shape, contextual paths, preserved unknown/repeated fields and deferred subitems.
They include `9007199254740993`, fractions, `handsHeld: 1.0`, invalid carry types,
invalid hands counts, wrong scalar types, duplicate modeled fields and malformed
or trailing JSON. Source packets use compiler AST spans; JavaScript never
reserializes their numbers or repeated payload keys. Probe transport converts
Rust numbers to strings before comparison, normalizing integral `1.0` against
the manual parser's `u8` hands count. Ordinary fields retain `serde_json::Number`;
exact original numeric token spelling and arbitrary precision are not promised.

The modular layout was replayed against the original PR30 source-model code at
`bc38e293105eee883ff488b4e2881eb20880f635`. That code was compiled in a private
scratch package with the same serde dependencies. All 4,580 corpus results and
all 18 adversarial results/rejection paths match exactly, including retained
legacy members. Generated declarations and parser bodies also have identical
tokens after accounting for module imports, visibility and formatting. This
checks behavior preservation through the layout change; the manual PR18
comparison above remains the separate source-modeling evidence.

## Policy and maintenance cost

All fields use `SourcePresence<T>` before Foundry defaults, including declaration-
required fields. This matches the manual source layer. The input records the
upstream required/optional/null differences; it does not turn them into persisted
JSON admission rules. Equipment's stricter declared usage/equipped presence
therefore shares the physical value structure. Small carry-type and hands-count
literal sets remain checked; usage stays an open string.

The emitter supports ordinary string/number/boolean values, selected object
intersections and homogeneous string/small numeric literal unions. Arrays,
recursion, index signatures, mixed unions, templates, unresolved/open types and
selected forbidden fields stop generation. Intersection constituents are checked
so resolved fields cannot hide an unsupported index signature. There is no
arbitrary-JSON fallback for an unsupported selected field.

The modular output is 242 Rust lines across two content modules and two indexes.
Handwritten support is about 350 Rust lines
for ordered source values, presence, diagnostics and slice composition, plus
about 450 TypeScript lines for selection/emission, snapshot loading/partitioning
and artifact formatting/freshness. Input metadata is 789 lines across a 20-line
manifest and two snapshots. Sampling, comparison probes and
tests add contributor validation code. The old corresponding model/parser
fragments are already compact; generation does not demonstrate a line-count
saving at this scale. Its demonstrated benefit is one declaration-driven edit
path and shared owners, with intentional source policy kept small and explicit.

Full generation remains unproven: arrays, keyed maps, recursion and broader
literal/open-string policy still need representative trials. Legacy modeling
requires a separate explicit decision; preserving legacy members is not typing
their semantics. This PR adds no database, metrics, record conversion, API/UI
contracts, production pipeline calls or product CLI commands.

## Reproduce

Use the pinned exported source and extraction described in the
[package instructions](../../dev-tools/source-contracts/README.md). Node 22+, Rust
and rustfmt are developer prerequisites. Freshness and regression checks use only
checked fixtures:

```sh
npm --prefix dev-tools/source-contracts ci --ignore-scripts
npm --prefix dev-tools/source-contracts run verify
cargo test -p atlas-ingest --test equipment_source_generation
```

For the manual comparison, create an isolated reference and private scratch
package. Run from the candidate repository root:

```sh
git worktree add --detach .worktrees/equipment-manual-reference \
  1ef0ab8fd13abf02e809a8ff47dd0ee9c3f97b29
mkdir -p scratch/equipment-comparison/manual/src
cp dev-tools/source-contracts/fixtures/manual-equipment-probe.rs \
  scratch/equipment-comparison/manual/src/main.rs
cat > scratch/equipment-comparison/manual/Cargo.toml <<EOF
[package]
name = "equipment-baseline-probe"
version = "0.0.0"
edition = "2024"
[workspace]
[dependencies]
atlas-ingest = { path = "$(pwd)/.worktrees/equipment-manual-reference/crates/atlas-ingest" }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
EOF
cargo build --manifest-path scratch/equipment-comparison/manual/Cargo.toml
cargo build -p atlas-foundry-model --example equipment_generation_probe
npm --prefix dev-tools/source-contracts run build
node dev-tools/source-contracts/dist/src/cli/sample-equipment.js --source scratch/pf2e \
  > scratch/equipment-comparison/packets.jsonl
scratch/equipment-comparison/manual/target/debug/equipment-baseline-probe \
  < scratch/equipment-comparison/packets.jsonl > scratch/equipment-comparison/manual.jsonl
target/debug/examples/equipment_generation_probe \
  < scratch/equipment-comparison/packets.jsonl > scratch/equipment-comparison/generated.jsonl
node dev-tools/source-contracts/dist/src/cli/compare-equipment.js \
  --packets scratch/equipment-comparison/packets.jsonl \
  --baseline scratch/equipment-comparison/manual.jsonl \
  --generated scratch/equipment-comparison/generated.jsonl \
  > scratch/equipment-comparison/report.json
```

Comparison exits 1 for any difference, including the six expected legacy ownership
differences. Inspect the report; there is no blanket exception or success flag
that hides them. Repeat both probes and comparison using
`dev-tools/source-contracts/fixtures/equipment-comparison.jsonl` for the 18 cases.
Remove the temporary reference worktree after recording results.
