# Source contract tooling

Offline developer tooling for investigating upstream PF2e declarations and authored metadata. These scripts do not run during normal Rust builds or change the ingest pipeline.

Use Node 22 or later. Install the pinned compiler and declaration dependencies, then run the fixture tests:

```sh
npm --prefix scripts/source-contracts ci --ignore-scripts
npm --prefix scripts/source-contracts test
```

`source-identity.mjs` identifies the source bytes independently of checkout location. It hashes `src`, `types`, the package and system manifests, compiler configuration and English localization. A real upstream checkout also reports its Git commit and dirty state; an exported archive reports no Git identity.

`extractTypeGraph(sourceRoot)` selects the persisted document kinds from `static/system.json` packs. Actor and Item use PF2e's aggregate Source unions; other kinds use the exported Foundry `KindSource` declaration. It follows the resolved source closure, including embedded documents, and compares Actor/Item `type` discriminator sets with `PF2ECONFIG.documentClasses`. This includes registered families with no records in the pinned corpus.

Built-in rules are selected from `RuleElements.builtin`. A memory-only TypeScript module applies Foundry's `SourceFromSchema` to each constructor's resolved `defineSchema()` return type. This handles inherited schema methods and generic runtime classes without executing upstream code or traversing prepared instance methods. Custom extension payloads remain visible as open source domains.

Registry discovery supports literal initializers in the upstream registry modules. Duplicate keys, dynamic entries and direct subsequent mutations are diagnosed. It does not simulate runtime registration or trace arbitrary aliases and mutations across the upstream application.

The pinned PF2e 6.12.4 extraction has 47 roots: five document kinds (all 24 Item and eight Actor families), plus 42 built-in rule schemas. It produces 3,084 graph nodes with no selected compiler errors or unsupported nodes. `complete` describes extraction of this selected closure, including the explicit serialization projections below. It does not establish corpus agreement or parser fidelity. New kinds, mismatched family sets, unresolved rules and unsupported registry syntax produce diagnostics instead of silently shrinking coverage.

`source-serialization.mjs` handles the inspected runtime declarations that occur inside source types. Its bounded projections retain original declarations and serialization metadata:

- `Predicate` extends `Array<PredicateStatement>` and `toObject()` returns `RawPredicate` with the same element type. Its persisted shape is an array. An unrelated runtime class, changed element type, or custom `toJSON` hook remains unsupported.
- `ModifierAdjustment.test`, `getNewValue`, and `getDamageType` are function-valued object properties. JSON serialization omits them. The graph retains their signatures and marks these names optional and forbidden in JSON, rather than accepting strings or objects as callbacks. New callbacks and changes to nonfunction values still produce diagnostics.
- `PickableThing.predicate` is declared as a runtime `Predicate`, but ChoiceSet passes it as one argument to `new Predicate(c.predicate ?? [])`. The graph derives the accepted statement-or-array input from that constructor's rest parameter. Other predicate fields retain their array shape.

A focused check of the pinned commit `4cbdaa37d6c33e9519561bae2c59a23e0288cbce` scanned 25,682 pack JSON files, including embedded records. It inspected 17,557 predicate arrays, 511 definition arrays, and 1,257 ChoiceSet predicates (including three single-statement objects). One unique upstream predicate fails the declared shape: `packs/classfeatures/revolutionary-innovation.json`, rule 0, choice 29 contains both `nor` and `not` in one object. Foundry's `StatementValidator` also requires one key for either operator, so this is a source-data discrepancy to carry into corpus comparison. The corpus has no nonempty custom modifiers; fixtures exercise their serialization instead. This focused investigation is not a full-corpus schema validator.

The declaration graph and trait catalog are separate implementation slices above this shared tooling base. Their combined command is a later layer. Rust generation and production adoption remain experiments tracked in the [source-contract backlog](../../docs/backlog/items/rust-source-contract-generation.md).
