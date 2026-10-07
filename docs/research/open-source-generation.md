# Open source domains and complete Item flags

Date: 2026-10-07

This report records PR33. The subsequent
[template/generic report](./template-source-generation.md) records the current
41-root compilation result and the authored-rule corpus discrepancies.

## Implemented model

The source emitter handles explicit any, unknown, object and non-nullish domains,
plus a single string index signature alongside named fields. Open values reuse
SourceValue while parsing checks the domain's JSON kinds. TypeScript object also
accepts arrays; an explicit empty TypeScript shape accepts non-null primitives.
These are declared open regions, not arbitrary-JSON fallbacks for unsupported
types. Structured open domains and indexed intersection constraints remain errors.
See [ADR 0037](../architecture/decisions/0037-open-and-indexed-source-values.md).

The callable shared Item source slice now uses generated ItemSourceFlagsPF2e,
replacing ItemGrantSlice, ItemGrantFields and their handwritten flags parser.
The generated flags module contains:

- A typed pf2e namespace with grantedBy, itemGrants and rulesSelections.
- Ordered dynamic pf2e entries carrying unknown JSON values.
- Ordered other namespaces, each a string-keyed map of unknown JSON values.
- A shared StringOrNumberOrObject enum for rule selection values. Its Object arm
  preserves arrays and objects; boolean/null map entries fail as declared.

Named fields retain missing/null/value before defaults. The index parser checks
non-null named values as well as dynamic entries. Dynamic keys reject duplicates;
explicitly open nested payloads retain repeated members. Declaration-forbidden
members remain raw additional data. This is source-shape modeling, not full
Foundry runtime admission or predicate evaluation.

The production ingest pipeline, artifact schema, metrics and UI continue through
their existing owners. No compatibility wrappers or partial handwritten flags
implementation remain.

## Source and generation evidence

- Integration base: ae5276a9e2fd8d1293eaff9b771c5d50ce2266b5, merged PR32.
- PF2e 6.12.4: 4cbdaa37d6c33e9519561bae2c59a23e0288cbce.
- TypeScript 5.9.3; saved complete extraction has 3,084 nodes and 47 roots,
  with source digest 6bf64da835272af22c53db729cb28d18a87e8dee999c11b154fb3e772befdeb8.
  This task reuses that pinned extraction; extraction code and source identity
  are unchanged.
- The refreshed selection has 791 nodes across six module snapshots. Nodes remain
  owned once. Complete flags have their own items/flags snapshot and Rust module.
- Generated production Rust totals 1,754 lines across nine files. Predicate,
  trait and physical/equipment Rust remain unchanged.
- Saved snapshots and generated outputs pass the full graph-mode freshness check.

## Corpus and fixture evidence

The expanded Item probe accepts all 98,651 sampled occurrences: 18,634 root Items
and 80,017 recursively embedded Items. All typed/presence/dynamic/additional
projections equal ordered raw source projections, with zero rejections or value
differences. Numbers retain their Rust JSON representation; open object members
remain ordered entries rather than being collapsed through JavaScript objects.

A separate value-discovery count, using JSON.parse only for counts rather than
fidelity, found 4,845 non-null flags objects, 4,664 non-null pf2e namespaces, 484
itemGrants fields, 536 grantedBy fields, 187 other namespace entries and 3,725
dynamic pf2e entries. There are no authored rulesSelections fields in this pin.
Its scalar/object/array alternatives therefore have fixture evidence, not corpus
evidence. Missing corpus families and source states retain the prior fixture coverage.

Verification includes 58 TypeScript tests on Node 22, the full Rust gate, eight
shared Item tests, 16 compiled generic-fixture tests and an independent raw-oracle
probe test. The probe test participates in the ordinary workspace test gate.
Fixtures check null and primitive domain boundaries, exact large integers,
repeated open members, duplicate modeled keys, escaped paths, index constraints
on named values, forbidden-member retention, undefined index unions, recursive
indexed structs, cross-module ownership, ambiguous broad union alternatives and
tuple-union guards containing open/null values. The last fixture verifies that
null uses Rust's unit variant while other JSON kinds use payload variants.

## Whole-portfolio diagnostic

The same per-root emission attempt now emits 27 of 47 roots, up from seven.
Twenty first blockers remain: 11 template domains and nine generic-instantiation
name collisions. The newly emitted full-root outputs have no portfolio compilation
or admission claim. First blockers do not inventory all constructs behind them.
Template strings and deterministic generic naming are the next generator work;
nullable collection unions and other unsupported forms remain visible.

## Reproduce

Follow the [source tooling instructions](../../scripts/source-contracts/README.md)
to export the pinned source and extract it completely, then refresh/check the
selected snapshots and Rust output. Export packs and static/system.json from
the same Git pin for corpus sampling. Run:

```sh
npm --prefix scripts/source-contracts run verify
scripts/verify.sh
cargo test -p atlas-ingest --test item_source_generation \
  --test recursive_generation_fixture --example item_generation_probe
cargo build -p atlas-ingest --example item_generation_probe
set -o pipefail
node scripts/source-contracts/dist/sample-items.js --source scratch/pf2e | \
  target/debug/examples/item_generation_probe > scratch/item-corpus-report.json
```

The corpus comparison exits 1 on any rejection or value difference; there is no
allowlist. Repeat the whole-portfolio diagnostic from the
[recursive source report](./recursive-source-generation.md) against the current
emitter to inspect the next first blockers.
