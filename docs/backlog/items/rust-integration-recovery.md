# Recover selected integration work onto main

Status: planned
Priority: source foundation first; selective product recovery later
Owner: unassigned
Last reviewed: 2026-10-05

## Direction and reference candidates

Start new work from main and recover useful integration behavior through small,
working PRs. Keep the integration branch and PR18 intact as references. This
inventory selects functional bundles, not commits to cherry-pick or whole crates
to copy. No recovery bundle below has been ported or tested on main yet.

Static inspection compared main `c7b74cbdc7c4df72b7c9a1c88c4f8dedbca959e7`
with integration `66d47fa9608c5f8c0b77e44efc85faade78d2b89`, using their
merge-base diff. PR18's separate reference head is
`1ef0ab8fd13abf02e809a8ff47dd0ee9c3f97b29`; its changes are not assumed merged.
Source evidence uses PF2e 6.12.4 at
`4cbdaa37d6c33e9519561bae2c59a23e0288cbce`.

Main already has the Rust CLI, app-service, web frontend, generated TypeScript
bindings, rich content, search and local-state foundation. Follow its
[overview](../../architecture/overview.md),
[runtime ownership](../../architecture/runtime.md) and
[artifact contract](../../architecture/artifact-contract.md).
The integration [parser-only decision](https://github.com/e-kosten/pf2e-atlas/blob/66d47fa9608c5f8c0b77e44efc85faade78d2b89/docs/architecture/decisions/0038-parser-only-source-modeling.md)
is useful recovery guidance, not a statement that main already implements it.

## Recovery inventory

### Declaration extraction and trait metadata — implement independently

The first PR follows [source generation and catalog research](./rust-source-contract-generation.md):
extract inspectable common/physical declarations and upstream trait/other-tag
metadata without changing ingest, storage or UI. It needs the pinned upstream
TypeScript environment, configuration and localization inputs; it does not need
integration's canonical records or artifact machinery. Validate deterministic
output, shared references, unresolved selected types, missing descriptions,
parameterized variants and an understandable change fixture. Rust generation
remains a later experiment; Atlas tags remain a separate product vocabulary.

### Source primitives, common Item and shared physical parsing — rework a bounded dependency bundle

Recover ordered duplicate-preserving JSON, authored numbers, presence, version
and contextual errors with the common/physical models and focused tests.
The [source DTO modules](https://github.com/e-kosten/pf2e-atlas/tree/66d47fa9608c5f8c0b77e44efc85faade78d2b89/crates/atlas-ingest/src/source/dto)
show the actual dependencies: `value`, `presence`, `diagnostic`, `version` and
`fields`; common parsing also uses `ItemType` and `ActorType`, currently housed
beside larger item/NPC parsers. The physical parser imports `CreatureSize` and
`CreatureFrequencyPeriod` from atlas-record; the value tree imports
`UnsupportedSourceShape`. These types are absent on main. Extract only the
necessary source or genuinely neutral leaves rather than importing canonical
family implementations to satisfy those names.

PR16/17 alone are not demonstrated portable commits. Reuse their
`common_item_source` and `physical_item_source` tests for root/actor contexts,
missing/null/empty states, duplicate members, recursive subitems, numeric fidelity
and contextual errors. Compare generated prototypes against these behaviors
before choosing manual ports or finished generated replacements. Keep the API
independently callable without adopting it into the production pipeline yet.

### Equipment/backpack/book/treasure refinements — retain PR18 as a source-only reference

The [PR18 family parser](https://github.com/e-kosten/pf2e-atlas/blob/1ef0ab8fd13abf02e809a8ff47dd0ee9c3f97b29/crates/atlas-ingest/src/source/dto/item_physical/family/parse.rs)
composes the common/physical parser and field helpers; its accompanying
`physical_item_families` tests exercise restrictions, legacy equipment fields,
recursive children and declaration-only books. Recover after the shared source
bundle or use as a comparison for generation. Forbidden-field absence and
pre-default presence need explicit fixtures. No observed book records were
available in the earlier comparison; declaration tests do not establish corpus
exercise. This does not complete the remaining physical families.

### Schema snapshots, diffs and field sampling — port after minimal tree support

The useful [offline discovery modules](https://github.com/e-kosten/pf2e-atlas/tree/66d47fa9608c5f8c0b77e44efc85faade78d2b89/crates/atlas-ingest/src/audit)
and `audit.rs` depend on the ordered source tree, existing manifest/pack loader
helpers and ingest errors. The required loader helpers already exist on main;
copying integration's changed production loader is not required by these imports.
Recover normalized path discovery, complete-baseline comparison, value counts
and bounded source references, plus `contracts/source-schema/v1` and focused
schema/value CLI tests. Preserve the coherent `atlas source schema|values|analyze`
grouping when the discovery surface lands; main's existing source analysis
behavior can be moved without recovering the canonical pipeline.

Validate duplicate keys, arrays/keyed maps, deterministic samples, missing packs,
incomplete-baseline rejection, display limits and schema drift exit behavior.
These are discovery tools, not field-owner ledgers or model-completeness gates.

### Generic browser layout, navigation and previews — port independently where contracts match

Compare the existing main components with integration's
[shared UI/layout code](https://github.com/e-kosten/pf2e-atlas/tree/66d47fa9608c5f8c0b77e44efc85faade78d2b89/web/atlas-ui/src/shared).
Responsive pane disclosure, Ant controls, navigation and the generic
`PreviewPopover` depend on React/Ant and existing shared primitives, not canonical
family data. Recover useful behavior against main's current interfaces and CSS.
Retain native Ant placement/collision handling rather than coordinate code.

The record-specific preview wrapper is different: it imports `RecordDetailPane`,
which consumes integration's surface, references and spell-selection API.
Rework its composition around main's existing record presentation rather than
copying those dependencies or introducing an old/new contract adapter. Validate
keyboard opening/Escape/focus return, late requests, navigation and responsive
light/dark layouts with focused tests and matching-candidate browser inspection.

### Canonical creature/hazard/spell semantics and regression fixtures — defer product adoption

Preserve useful [atlas-record semantics](https://github.com/e-kosten/pf2e-atlas/tree/66d47fa9608c5f8c0b77e44efc85faade78d2b89/crates/atlas-record/src)
as design/test references: intrinsic versus occurrence identity, repeated embedded
uses, owned content, typed unsupported states and spell form/heightening resolution.
Later ports need explicit ingest conversion, reader/writer codec and projection
dependencies; source parser completion alone supplies none of these guarantees.
Select meaningful fixtures and mutation tests, not entire historical test trees.
Validate identity/order, sibling preservation, form arithmetic and unavailable
cases before bounded writer-to-public-reader tests in the adopting product slice.

### Curated record UI and encounter runtime — defer until backend slices are reviewed

The [creature/hazard/spell surfaces](https://github.com/e-kosten/pf2e-atlas/tree/66d47fa9608c5f8c0b77e44efc85faade78d2b89/web/atlas-ui/src/shared/records)
consume generated family DTOs composed by `atlas-app-service::surface` and
`hazard_surface` from canonical `RecordBody` values. They are not standalone
frontend copies. Recover each family with its reviewed backend contract, binding
freshness, focused UI tests and visual validation. Spell compact projection
currently computes catalog/content that its compact renderer does not consume;
review profile selection when recovering it rather than preserving that cost.

Encounter features additionally depend on canonical mechanics/occurrences,
condition adjustments, app-service workflows and atlas-local-state baselines,
reset and spell-resource schema. Recover only after the corresponding static
semantics, then test independent participants, spend-owner validation, atomic
reset and unavailable inputs. Diagnostic objects immediately discarded by
`encounters::mechanics::into_public_runtime` deserve simplification; preserve
unsafe-value suppression and user-facing automation limitations.

### Artifact and validation changes — re-evaluate, do not port wholesale

Integration's [artifact implementation](https://github.com/e-kosten/pf2e-atlas/tree/66d47fa9608c5f8c0b77e44efc85faade78d2b89/crates/atlas-index/src/artifact)
combines canonical codecs/tables with publication pairs, manifests and immutable
generations. Later storage decisions need only their demonstrated requirements;
independent extraction/source parsing requires none of that machinery. Separate
real transaction/publication/reader safety from historical agent evidence.
Validate accepted storage changes through fault/rebuild compatibility tests and
bounded end-to-end production checks when those boundaries actually change.

Earlier AO-bound exporter/approval metadata is historical reference, not a
recovery dependency; the old `e3_sample_export.rs` is already absent at this
integration revision. Reassess remaining optional validation plumbing and useful
behavior tests without restoring receipts, authorization wrappers or corpus
replay gates. This inventory neither deletes integration files nor establishes
that artifact publication receipts are equivalent to AO process receipts.

## Sequence and completion

Land independent extraction/catalog tooling first. Review its output before Rust
generation; then choose a bounded source primitive/common/physical recovery or
generated replacement, followed by discovery and remaining source families.
Generic browser improvements may proceed independently. Canonical conversion,
storage, curated family UI and encounter expansion remain later reviewed work.

Each recovery PR must work on main with its dependency closure, useful behavior
and risk-appropriate validation demonstrated. Remove superseded implementations
within an adopting slice; no compatibility wrappers or parallel production models.
Keep crate ownership, update current docs and regenerate actual API bindings when
affected. Static import inspection proves the dependencies named here, not a
complete compilable port, preservation of all behavior, performance or UI
acceptance. No tests or ports were performed for this inventory.
