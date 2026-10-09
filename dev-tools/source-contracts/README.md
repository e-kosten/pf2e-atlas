# Source contract discovery

Offline developer tooling for investigating upstream PF2e declarations and authored trait metadata, and generating the complete pinned Rust source portfolio. It does not adopt generated parsing into the ingest pipeline, artifact, API or UI.

## Run

Use Node 22 or later and Rust with rustfmt for generation/freshness and compiled
parser fixture checks. Prepare Rust dependencies once; fixture probes build offline.
From the Atlas repository root:

```sh
npm --prefix dev-tools/source-contracts ci --ignore-scripts
cargo fetch --locked
npm --prefix dev-tools/source-contracts run verify
```

The package is private contributor tooling. `build` replaces ignored `dist/` and emits JavaScript;
`typecheck` checks implementation and tests without emitting; `test` builds and runs
the fixture tests; `verify` runs both checks, including synthetic Rust fixture freshness. `extract` builds before executing
the compiled entry point. Fixtures stay in the source package. Public discovery
contracts live in `src/contracts.ts`; compiler-internal access is bounded in
`src/discovery/compiler-types.ts`. Neither this package nor its dependencies ship with Atlas.

Command argument parsing and exit behavior live in `src/cli/`. Compiler extraction
and catalog discovery live in `src/discovery/`; graph selection and Rust emission
live in `src/generation/`; sampling and probe comparisons live in `src/comparison/`.
Tests live in `tests/`, synthetic inputs in `fixtures/`, and the small source identity
in `source-pin.json`. Large extraction outputs belong in ignored caches. The build mirrors these directories under ignored `dist/`.

## Source portfolio generation

`generate` emits all 47 extracted roots: Actor/Item sources across eight/24
registered families, JournalEntry, Macro, RollTable, and 42 specific rule models.
The refresh recipe in `src/generation/portfolio-selection.ts` applies the proven
authored-input policies and retains the existing independently callable Item,
physical/equipment and predicate components. Full sources include fields that
those narrower slices intentionally leave as additional data.
Unsupported constructs stop generation. SourcePresence keeps missing/null/value
before defaults; explicit open domains and undeclared/forbidden members retain
ordered source values. Original declarations and serialization provenance remain
in the ignored generation input alongside authored projections. Parsing is separate from
Foundry admission, preparation, canonical conversion and production ingest.

From the repository root, check or regenerate directly from the pinned source:

```sh
npm --prefix dev-tools/source-contracts run generate -- --check
npm --prefix dev-tools/source-contracts run generate
```

The command uses `source-pin.json` and locked TypeScript/declaration dependencies.
A cold cache fetches the immutable upstream commit using Git and exports its
source inputs using tar. Source exports and extraction evidence live under ignored
`.cache/source-contracts/<commit>/`; the application does not use them.
Source bytes/version/file count and compiler identity are checked before generation.
Every invocation extracts afresh, so a stale graph cannot authorize Rust output.

Use `--source PATH` to seed the cache from a matching local checkout/export
without fetching. Its source bytes must match the pin; the command copies the
needed inputs into an isolated export and leaves the original dependencies alone.
`--cache-dir PATH` chooses another tool-owned cache; `--out-dir PATH` chooses
another generated Rust directory. Paths are relative to the caller's directory.
Warm exports allow offline regeneration after npm dependencies are installed.
Corrupt source caches fail visibly; remove the ignored cache to fetch a clean export.

Ordinary `verify` runs offline synthetic fixture tests. CI separately runs:

```sh
npm --prefix dev-tools/source-contracts run verify-generated
```

This re-extracts the pinned source, compares every generated Rust file, and exercises
whole-portfolio ownership, declaration mutations and unsupported shapes. Offline
fixtures cover output preflight. Missing/obsolete generated files fail freshness. Regeneration removes
only obsolete tool-owned files and refuses unmanaged files or symlinks. Formatting
and preflight finish before writes. A normal Rust build needs neither Node nor
upstream source. See [ADR 0034](../../docs/architecture/decisions/0034-source-generation-layout.md).

To update upstream, run `extract` against the proposed source, review the graph
and corpus results, update the pin's commit/version/digest/count/compiler/rule keys
and dependency lock as needed, then regenerate Rust. Schema comparisons can extract
two revisions into ignored directories and diff their graphs. No large intermediate
graph is committed.

The emitter resolves one complete selection and assigns shared value owners globally.
Outputs include common Item/Actor/physical components, `items/families/*`,
`actors/families/*`, `documents/*`, `rules/*` and their indexes. Family source and
system roots reserve their modules before traversing embedded references; shared
shapes still have one owner. Large object union payloads are boxed to bound enum
layout without changing serialized values. The public Rust value and selected
root-parser namespace is `atlas_foundry_model::generated`.
Byte entry points in `atlas_foundry_model` parse the five document kinds
and keyed `RuleSource`. Item.rules follows upstream's generic RuleElementSource;
use the specific rule parser separately. Existing field-level slice entry points
remain independently callable.
Only modules with definitions or existing children are emitted. Future families
extend this organization; [ADR 0034](../../docs/architecture/decisions/0034-source-generation-layout.md)
records the ownership and layout rules.

The maintained selection derives explicit open trait-array policies from declaration identity. These arrays retain
their declared vocabulary nodes in input but generate string values. A policy
target that becomes a nonstring array rejects generation. Ordinary string arrays
may share the same vector owner. Current finite other-tag enums, rarity, license
and grant deletion behavior remain checked. Declaration-forbidden persisted
members remain additional data before defaults. Pure string-keyed maps preserve
typed values and authored order, rejecting repeated modeled keys. Nullable
entries use inline `Option<T>`; null-only entries use `()` and a null-only parser.
Undefined array/fixed-tuple entries persist as null, while undefined index values
permit omitted keys. Explicit nullable index unions admit present null values;
undefined alone does not. Optional/rest tuples, nullable value roots, alias-only
recursion, multiple index signatures and key domains other than string/number
remain unsupported. Anchored recursion,
mixed unions and fixed tuples are supported. Union identity requires exactly one
shape candidate; required keys count even with null/invalid payload, and required
literal discriminants retain nullable state. Additional partial operator keys
remain additional values when they do not identify another complete arm. This is
declaration-shaped source modeling, not full Foundry runtime admission. A shared
required literal field with pairwise disjoint domains identifies an object arm
without requiring the other fields before defaults. Missing/overlapping tags do
not justify selecting an arbitrary arm.

Anonymous unions of complete scalar types have member-derived names in fixed
String, Number, Boolean order (`StringOrNumber`, `StringOrBoolean`,
`NumberOrBoolean`, `StringOrNumberOrBoolean`). Complete true/false pairs represent
Boolean; restricted literal alternatives retain their checks. New owners prefer
upstream declared names, while equivalent shapes reuse existing owners. Naming
collisions fail explicitly. Generic instantiations use the declaration plus simple
named arguments, or the field/root context for complex arguments. Anonymous tuple
fields and union payloads use inline Rust tuples and share a private parser;
declared or selected root names retain
aliases. These source types are inputs to later ingest interpretation, rather
than prescribed application or storage models.

Explicit any/unknown declarations retain JSON in SourceValue. TypeScript object
accepts arrays/objects; an explicit empty shape accepts all non-null JSON.
Their parsing primitives enforce these boundaries without inventing nested fields.
Atomic unions extend member-derived names to these domains, such as
`StringOrNumberOrObject`. Unsupported declarations do not become arbitrary JSON.
Named-plus-indexed objects retain SourcePresence named fields, typed ordered
`indexed_fields`, and raw `additional_fields` for declaration-forbidden members.
The string index parser also checks non-null named values. Named missing/null states
remain pre-default facts; dynamic entries use the index's actual null constraints.
Undefined in an index value union allows absent keys, not undefined JSON entries.
Object intersections carry compiler-resolved fields and index signatures for
the complete intersection; narrowed/merged value constraints generate through
the same struct and pure-map representations. Re-extract older graphs without
intersection index metadata. Numeric indices generate structs whose typed ordered
entries retain canonical TypeScript numeric names as strings; other names remain
additional source data. Numeric named fields satisfy both their field and index
constraints. Pure string maps retain SourceMap; pure numeric maps have a nominal
struct owner, which also anchors recursive numeric maps. Multiple indices and
other key domains remain unsupported. See
[ADR 0037](../../docs/architecture/decisions/0037-open-and-indexed-source-values.md).

Rust field names support leading underscores and map punctuation, digit-leading
names and reserved path keywords while retaining source keys for parsing and
explicit serde renames for these mappings. Name collisions, including generated
retention slots, stop generation. See
[ADR 0035](../../docs/architecture/decisions/0035-source-value-generation-policy.md).
The [collection report](../../docs/research/collection-source-generation.md)
records nullable-entry support. The
[document report](../../docs/research/document-source-generation.md) records
complete 47-root generation/compilation and the Actor/Item corpus baseline.

### Maintained Rust corpus comparison

```sh
npm --prefix dev-tools/source-contracts run compare-portfolio -- \
  --source scratch/pf2e \
  --baseline dev-tools/source-contracts/fixtures/portfolio-corpus-baseline.json \
  --out scratch/portfolio-comparison
```

This re-extracts the pinned source, checks Rust freshness, builds the actual atlas-foundry-model
`source_portfolio_probe` example offline, and samples all five document kinds
plus every specific Item rule. Output includes raw `packets.ndjson`,
`results.ndjson` and `comparison.json` with per-root counts, every failure context,
fidelity and a corpus/rejection baseline comparison. No scratch parser substitutes
for the maintained crate. Unknown rule keys and document kinds fail visibly.
The command uses the repository Cargo target directory (or CARGO_TARGET_DIR).

Generated unions now serialize with explicit `$variant`/`$value` tags. The
independent declaration oracle checks authored payloads; snapshot checks assert
exact typed variants. The codec validates generated source identity plus format
version and never reparses retained source. `parse_source_value` is the explicit
ordered authored-JSON reader. Filesystem loading remains in atlas-ingest.


The saved baseline detects changed corpus bytes, counts and first-error outcomes.
It is comparison evidence, not an ingest allowlist. Matching known rejections
still returns exit 1; fidelity failures and baseline changes also fail. A failed
run invalidates its previous report. Full Foundry admission remains unexecuted.
See the [maintained portfolio report](../../docs/research/maintained-source-portfolio.md).

For the separate raw-retention policy, run the same command with `--admission`:

```sh
npm --prefix dev-tools/source-contracts run compare-portfolio -- \
  --source scratch/pf2e --admission \
  --baseline dev-tools/source-contracts/fixtures/portfolio-admission-baseline.json \
  --out scratch/portfolio-admission
```

This measures the maintained `admit_*` APIs. Counts distinguish raw retention,
typed models, models without diagnostics (`fullyTyped`), partial models and
raw-only roots/rules. `fullyTyped` means representation parsing without recovery;
it does not imply required runtime defaults or complete mechanics. The report
keeps every diagnostic and checks unchanged raw values plus typed-field and
invalid-field fidelity. A malformed collection retains its complete raw value;
it is not shortened. Specific rule rejection excludes the whole rule from typed
interpretation while retaining its raw payload. Diagnostics do not alone make
this mode fail; rejected envelopes, fidelity failures or a mismatched optional
admission baseline do. Strict comparison remains unchanged, and its rejection
baseline must not be passed to admission mode. These are callable source APIs;
the production index pipeline does not yet consume them.

See [ADR 0041](../../docs/architecture/decisions/0041-source-admission-and-field-retention.md)
for field boundaries, root retention and constructor-input evidence.

### Schema/authored document comparison

```sh
npm --prefix dev-tools/source-contracts run compare-documents -- \
  --source scratch/pf2e \
  --graph scratch/source-extraction/type-graph.json \
  --summary scratch/source-extraction/summary.json \
  --out scratch/document-comparison
```

Use a complete source export including packs and a complete extraction of the
same source bytes. `cargo fetch --locked` prepares Rust dependencies; the scratch
probe builds offline. The command generates and compiles **all extracted roots
together**, then samples every root Actor and root/embedded/nested Item. The
existing open-string policy applies to every Actor/Item `system.traits.value`
identifier array; generation still checks that its declaration is a string vocabulary.
An explicit `never[]` keeps its empty-array constraint.
Rarity, other tags and other small vocabularies keep their declared constraints.

Output contains `comparison.json`, raw `packets.ndjson`, `authored-graph.json`,
and separate `schema/` and `authored/` generated modules/probes with line-by-line
`results.ndjson`. Both profiles compile all roots. The report's top-level counts
describe authored input; `schema` retains declaration-shaped results and
`transition` records recovered occurrences and acceptance regressions. `changes`
identifies document field policies and upstream evidence; `ruleChanges` describes
the emitted authored rule roots. Source/graph/corpus identity, unobserved
registered families, fidelity, grouped examples and every first error remain
available.
Raw packets stream to disk; large numeric tokens and repeated payload keys are
parsed by Rust without a JavaScript numeric round trip. This command and
`compare-rules` share one compiler/probe implementation.

Rejections, acceptance regressions or fidelity failures produce exit 1. Compilation/command failures
also fail; the command never repairs or silently excludes an occurrence. An
Actor rejected because of an embedded Item is counted alongside that Item's
separate occurrence. A first error can conceal further errors in that packet.
Accepted values are checked against raw source for modeled values, presence,
collection order and retained additional data. The authored profile preserves
source-backed sentinels, spell area number/string values and initiative statistic
slugs. Spell override and fixed heightening systems use recursive object patches;
arrays/tuples remain complete replacements. Complete source unions outside patch
contexts retain their tag/shape constraints. Parsing does not execute Foundry
cleaning, migrations, contextual spell merging or runtime admission. Item rules
use upstream's generic `RuleElementSource`; run `compare-rules` to exercise the
specific rule models. Known conflicts stay visible. See the
[authored document report](../../docs/research/authored-document-source.md).
Production ingest and product/storage models are unchanged.

The callable Item slice now uses the generated full flags declaration, including
`grantedBy`, `itemGrants`, `rulesSelections` and module namespaces. Open payloads
remain source evidence for later interpretation; they do not automatically become
product fields or database columns.

Template domains with arbitrary string interpolations generate String aliases
whose parsers check literal prefixes, intermediate fragments and suffixes.
Interpolations include empty strings, line breaks and Unicode. Template/literal
string unions share one combined string domain, including overlapping patterns.
Other interpolation domains fail explicitly. Mixed union guards and required
object discriminants retain template constraints. See
[ADR 0038](../../docs/architecture/decisions/0038-source-templates-and-generic-names.md)
and the [portfolio comparison](../../docs/research/template-source-generation.md).

[ADR 0036](../../docs/architecture/decisions/0036-recursive-source-unions.md) defines
the recursive and union policies; [ADR 0035](../../docs/architecture/decisions/0035-source-value-generation-policy.md)
records these policies.

For shared Item fidelity against a pinned source export:

```sh
npm --prefix dev-tools/source-contracts run build
cargo build -p atlas-foundry-model --example item_generation_probe
npm --prefix dev-tools/source-contracts run generate -- --source scratch/pf2e --check
set -o pipefail
node dev-tools/source-contracts/dist/src/cli/sample-items.js --source scratch/pf2e | \
  target/debug/examples/item_generation_probe \
    .cache/source-contracts/4cbdaa37d6c33e9519561bae2c59a23e0288cbce/extraction/generation-input.json \
    > scratch/item-corpus-report.json
```

The sampler uses raw AST spans for root and recursively embedded Item sources in
Item/Actor packs; additional payload numbers and repeated members are preserved.
The Rust probe compares typed values, presence and ordered additional members
against raw source projections, using freshly extracted selection metadata from the ignored cache. Numbers
remain in Rust until comparison transport. It reports per-family counts and
exits 1 on any rejection or value difference. This proves selected-slice fidelity,
not complete declaration coverage. See the
[shared Item report](../../docs/research/shared-item-source-generation.md).

For predicate value discovery and fidelity:

```sh
npm --prefix dev-tools/source-contracts run build
cargo build -p atlas-foundry-model --example predicate_generation_probe
set -o pipefail
node dev-tools/source-contracts/dist/src/cli/sample-predicates.js --source scratch/pf2e | \
  target/debug/examples/predicate_generation_probe > scratch/predicate-corpus-report.json
```

The sampler scans authored `predicate` fields, rule `definition` fields (including
nested exceptions), ChoiceSet `choices.filter`, CraftingAbility `craftableItems`,
RollOption `disabledIf` and predicate-valued FlatModifier/SubstituteRoll
`removeAfterRoll`. Only ChoiceSet `choices[i].predicate` uses statement-or-array
input. Malformed non-array values at array-only paths are emitted as discrepancies.
Ten pinned ItemAlteration definitions are labeled legacy discovery; sampling
does not establish typed family ownership. The known revolutionary-innovation
predicate conflict makes the pin's comparison exit 1: inspect the report rather
than suppressing it. All other 18,512 selected occurrences have equal typed values.
See the [recursive report](../../docs/research/recursive-source-generation.md).

Synthetic regression graphs live in `fixtures/recursive-fixture.ts`; their generated
test-only Rust files under `crates/atlas-foundry-model/tests/fixtures/source_model/generated`
compile against the actual source primitives. Package tests verify file freshness;
Rust tests execute mixed recursive layouts, empty tuples, literal roots, nullable
discriminants and optional-arm ambiguity. To refresh after an intentional fixture
change, build the package and use `generateRustModules(recursiveFixture())` plus
`formatRust()` to write that directory, then run both package and Rust verification.

`sample-equipment --source PATH` emits JSONL source packets for root equipment and
direct Actor.items. Payload spans preserve authored numeric tokens and duplicate
keys. `compare-equipment --packets PATH --baseline PATH --generated PATH` compares
the Rust probes' JSONL results, reports counts/presence and full differences, and
exits 1 on any difference. npm commands build first; for redirected JSONL run
`npm --prefix dev-tools/source-contracts run build` once and invoke
`node dev-tools/source-contracts/dist/src/cli/sample-equipment.js` directly so npm's
progress output does not contaminate packets.

The [comparison report](../../docs/research/equipment-source-generation.md) records
the exact source/manual candidates, 4,580 corpus occurrences, legacy differences,
maintenance cost, limitations and reproducible probe instructions. Sampling and
comparison are private experiment tooling, not additions to either Rust CLI.

`sample-rules --source PATH` emits JSONL packets for direct system.rules of every
recursively sampled root/embedded Item. Missing/duplicate rule keys, non-object
rules and non-array rule lists fail visibly; raw tokens and additional duplicates
remain intact. Use direct Node execution after building when redirecting JSONL.

`compare-rules --source PATH --graph PATH --summary PATH --out PATH` builds scratch Rust portfolios for extracted schema
shapes and the bounded authored scalar-or-array projection. Re-extract first:
graph and summary must identify the same source bytes. Trait identifier policies
derive from the supplied declaration graph. Nested IWR objects reuse
the authored projection through matching compiler declaration provenance;
`sharedIwrChanges` lists these owners without broadening unrelated arrays.
`valueChanges` records ChoiceSet predicate optionality, DamageDice override
expression shapes and the nested BattleForm strike base-type string projection.
Direct Strike base types retain their extracted vocabulary. Strike scalar traits follow the same
explicit vocabulary policy as their array form. Generic unions select anchored
shapes before open/optional-only fallbacks; malformed anchored payloads retain
their nested errors. Declaration-forbidden keys exclude union arms while remaining
additional data in ordinary standalone object parsing.
The command writes
per-occurrence results and `comparison.json`, counts every rejection/unmodeled
key, and checks typed value/presence and ordered additional-member fidelity in
Rust. Rejections, fidelity loss or acceptance regressions exit 1. It requires
locally cached Cargo dependencies for offline builds. Runtime admission is
reported separately as not executed; parser rejection is not upstream-invalid
evidence. See the [authored-rule report](../../docs/research/authored-rule-source.md)
and [ADR 0039](../../docs/architecture/decisions/0039-authored-rule-inputs.md).

The source directory needs upstream `src`, `types`, `package.json`, `tsconfig.json`, `static/system.json` and `static/lang/en.json`. The compiler must also resolve the upstream declaration dependencies. For a clean exported source directory in `scratch/pf2e`, use:

```sh
ln -s ../../dev-tools/source-contracts/node_modules scratch/pf2e/node_modules
npm --prefix dev-tools/source-contracts run extract -- \
  --source scratch/pf2e --out scratch/source-extraction
```

Use the dependencies appropriate to the source version being inspected. The lockfile here pins TypeScript 5.9.3 and the declaration dependencies used to investigate PF2e 6.12.4 at `4cbdaa37d6c33e9519561bae2c59a23e0288cbce`. Do not replace an existing upstream dependency installation with a symlink. Inspect dependencies when changing the source pin.

Output files are `type-graph.json`, `trait-catalog.json` and `summary.json`. Output belongs outside the source input directories. The command prints the summary. `--strict` exits 1 when either extraction is incomplete, after writing the available discovery output and diagnostics. Without it, partial discovery exits 0 with `complete: false`. Invalid arguments, unreadable required inputs and fatal errors exit 1.

## Declaration graph

`source-serialization.ts` handles the inspected runtime declarations that occur inside source types. Its bounded projections retain original declarations and serialization metadata:

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

The PF2e 6.12.4 pin has 47 roots: five document kinds (all 24 Item and eight Actor families), plus 42 built-in rule schemas. It produces 3,085 graph nodes with no selected compiler errors or unsupported nodes and 226 unrelated full-project diagnostics. The graph and trait catalog report `complete`; normal discovery and `--strict` both exit 0 on this pin. The explicit serialization projections above are part of the supported extraction boundary. The malformed corpus predicate remains a separate data discrepancy, not an extraction failure.

The summary records a source digest, source version, Git commit/dirty state when the source is a checkout, TypeScript version and dependency lock digest. Archives have no Git identity. The source digest covers relative names and bytes of `src`, `types`, the package and system manifests, compiler config and English localization; it is independent of checkout location. The lock digest identifies the intended environment; it does not attest an arbitrary external `node_modules` installation.

Save outputs for each source revision and use a normal JSON diff to discover field, vocabulary and metadata changes. Extraction output is developer evidence, not a runtime admission gate or a tracked generated Rust model. See the [source-contract experiment](../../docs/backlog/items/rust-source-contract-generation.md) and [selective recovery inventory](../../docs/backlog/items/rust-integration-recovery.md) for the remaining work.
