# Source contract discovery

Offline developer tooling for investigating upstream PF2e declarations and authored trait metadata. This changes no Rust parser, ingest pipeline, artifact, API or UI. Rust generation remains a separate experiment.

## Run

Use Node 22 or later. From the Atlas repository root:

```sh
npm --prefix scripts/source-contracts ci --ignore-scripts
npm --prefix scripts/source-contracts test
```

The source directory needs upstream `src`, `types`, `package.json`, `tsconfig.json`, `static/system.json` and `static/lang/en.json`. The compiler must also resolve the upstream declaration dependencies. For a clean exported source directory in `scratch/pf2e`, use:

```sh
ln -s ../../scripts/source-contracts/node_modules scratch/pf2e/node_modules
node scripts/source-contracts/extract.mjs \
  --source scratch/pf2e --out scratch/source-extraction
```

Use the dependencies appropriate to the source version being inspected. The lockfile here pins TypeScript 5.9.3 and the declaration dependencies used to investigate PF2e 6.12.4 at `4cbdaa37d6c33e9519561bae2c59a23e0288cbce`. Do not replace an existing upstream dependency installation with a symlink. Inspect dependencies when changing the source pin.

Output files are `type-graph.json`, `trait-catalog.json` and `summary.json`. Output belongs outside the source input directories. The command prints the summary. `--strict` exits 1 when either extraction is incomplete, after writing the available discovery output and diagnostics. Without it, partial discovery exits 0 with `complete: false`. Invalid arguments, unreadable required inputs and fatal errors exit 1.

## Declaration graph

`source-serialization.mjs` handles the inspected runtime declarations that occur inside source types. Its bounded projections retain original declarations and serialization metadata:

- `Predicate` extends `Array<PredicateStatement>` and `toObject()` returns `RawPredicate` with the same element type. Its persisted shape is an array. An unrelated runtime class, changed element type, or custom `toJSON` hook remains unsupported.
- `ModifierAdjustment.test`, `getNewValue`, and `getDamageType` are function-valued object properties. JSON serialization omits them. The graph retains their signatures and marks these names optional and forbidden in JSON, rather than accepting strings or objects as callbacks. New callbacks and changes to nonfunction values still produce diagnostics.
- `PickableThing.predicate` is declared as a runtime `Predicate`, but ChoiceSet passes it as one argument to `new Predicate(c.predicate ?? [])`. The graph derives the accepted statement-or-array input from that constructor's rest parameter. Other predicate fields retain their array shape.

A focused check of the pinned commit `4cbdaa37d6c33e9519561bae2c59a23e0288cbce` scanned 25,682 pack JSON files, including embedded records. It inspected 17,557 predicate arrays, 511 definition arrays, and 1,257 ChoiceSet predicates (including three single-statement objects). One unique upstream predicate fails the declared shape: `packs/classfeatures/revolutionary-innovation.json`, rule 0, choice 29 contains both `nor` and `not` in one object. Foundry's `StatementValidator` also requires one key for either operator, so this is a source-data discrepancy to carry into corpus comparison. The corpus has no nonempty custom modifiers; fixtures exercise their serialization instead. This focused investigation is not a full-corpus schema validator.

The default document kinds come from `static/system.json` packs. Actor and Item use PF2e's aggregate Source unions; other kinds use the exported Foundry `KindSource` declaration. The graph follows embedded source documents and compares Actor/Item `type` discriminator sets with `PF2ECONFIG.documentClasses`, including registered families with no corpus records. It resolves inherited fields, shared references, recursion, generic instantiations, discriminated unions, keyed maps, tuples and field presence (`optional`, `null`, `never`). Node identities and source locations are relative to the inputs; compiler-local numeric identities are not persisted.

Built-in rules come from `RuleElements.builtin`. A memory-only TypeScript module applies Foundry's `SourceFromSchema` to each constructor's resolved `defineSchema()` return type. This handles inherited schema methods and generic runtime classes without executing upstream code or traversing prepared instance methods. Custom extension payloads remain visible as open source domains.

Registry discovery supports literal initializers in the upstream registry modules. Duplicate keys, dynamic entries and direct subsequent mutations are diagnosed. It does not simulate runtime registration or trace arbitrary aliases and mutations across the upstream application. New kinds, mismatched family sets, unresolved rules and unsupported registry syntax produce diagnostics instead of silently shrinking coverage.

Unsupported or unresolved source values produce diagnostics and mark the selected closure incomplete. Explicit authored `any` and open domains remain visible rather than being narrowed. Generic schema arguments are provenance; resolved properties determine the serialized source-value closure. Compiler diagnostics touching visited declarations are separated from unrelated project diagnostics.

`extractTypeGraph(sourceRoot, { roots, maxNodes })` is the offline library entry point; custom roots use `{ file, name }`. The default closure limit is 4,000 nodes. Support is bounded by the constructs covered by the implementation and fixtures, rather than being a general TypeScript-to-schema translator.

## Trait catalog

`extractTraitCatalog(sourceRoot)` interprets the authored catalog expressions without evaluating upstream code or starting Foundry. It retains identifiers, label/description keys, English text, catalog memberships, exposure/export flags and source locations. Traits, otherTags and rarity have distinct namespaces. Missing metadata remains `null` with localization warnings where a known key lacks text.

Direct family evidence is recorded separately for runtime getters, declaration vocabulary and catalog key types. These can disagree. Folder names are upstream names; inherited or dynamic runtime applicability is not inferred. Authored descriptions retain their macros and prose; no mechanics or implication relationships are generated.

## Interpreting completeness and changes

`complete` means supported extraction of the selected declaration closure and authored catalog expressions. It does not mean production models exist for every family, the whole upstream project type-checks, declarations agree with every corpus record, or every trait has metadata.

The PF2e 6.12.4 pin has 47 roots: five document kinds (all 24 Item and eight Actor families), plus 42 built-in rule schemas. It produces 3,084 graph nodes with no selected compiler errors or unsupported nodes and 226 unrelated full-project diagnostics. The graph and trait catalog report `complete`; normal discovery and `--strict` both exit 0 on this pin. The explicit serialization projections above are part of the supported extraction boundary. The malformed corpus predicate remains a separate data discrepancy, not an extraction failure.

The summary records a source digest, source version, Git commit/dirty state when the source is a checkout, TypeScript version and dependency lock digest. Archives have no Git identity. The source digest covers relative names and bytes of `src`, `types`, the package and system manifests, compiler config and English localization; it is independent of checkout location. The lock digest identifies the intended environment; it does not attest an arbitrary external `node_modules` installation.

Save outputs for each source revision and use a normal JSON diff to discover field, vocabulary and metadata changes. Extraction output is developer evidence, not a runtime admission gate or a tracked generated Rust model. See the [source-contract experiment](../../docs/backlog/items/rust-source-contract-generation.md) and [selective recovery inventory](../../docs/backlog/items/rust-integration-recovery.md) for the remaining work.
