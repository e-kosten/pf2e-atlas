# Source research and pinned data

Read this guide when changing the source pin, generated models, corpus probes or
embedding/storage diagnostics. Commands run from the repository root. These
procedures are separate from ordinary frontend iteration.

## Pinned PF2E Data

`atlas setup --path-mode repo` acquires the model-aligned revision into
`.cache/pf2e/<commit>`. The canonical pin is
`dev-tools/source-contracts/source-pin.json`; runtime identity is generated from it.
Global mode uses `pf2e-atlas/pf2e/<commit>` under the platform cache directory.
The cache is detached and immutable: local edits or a different revision are
reported, never overwritten. Cold acquisition requires Git and network access;
warm check/offline use does not fetch.

A separate `vendor/pf2e` checkout may be used for developer diagnostics or passed
explicitly with `--source`. Atlas never pulls or checks out supplied directories.
Existing workspaces retain their saved source selection. To move one to the
managed repo-mode source without touching the old checkout or saved state:

```bash
atlas setup --path-mode repo --source .cache/pf2e/4cbdaa37d6c33e9519561bae2c59a23e0288cbce
```

PF2e system sources must declare the supported version for product builds.
Custom module versions are not interpreted as PF2e versions. Developer
`atlas-dev source load` remains available for diagnosing unsupported inputs.
Changing the pin requires regeneration and corpus/consumer validation; it is not
an ordinary data refresh. See ADR0052.

## Source declaration research

The shared HTML interpretation API is in `atlas-record::source_content`. Its
tests include typed generated-source traversal with an owned Item and invalid
neighboring field. The developer example can prepare an explicitly inventoried
HTML-field JSONL sample with supplied localization and a simple root-record
lookup index:

```bash
cargo run -p atlas-record --example source_content_probe -- HTML-fields.jsonl locale.json records.jsonl output.jsonl
```

Input field packets contain `owner` (`pack.id`, optionally with an owned suffix),
`field` (dotted source field) and `markup`; record-index packets contain `id`
(`pack.id`) and `name`. These are probe inputs, not product CLI or artifact
contracts. The probe explicitly uses source-inspection audience settings and
snapshot-local child locators. The output is not a browser/Foundry acceptance
result or an audience-safe application response. See
[ADR 0044](../architecture/decisions/0044-shared-source-content-interpretation.md).

Offline declaration and trait metadata tooling lives in [dev-tools/source-contracts](../../dev-tools/source-contracts/README.md). Its pinned Node dependencies, strict TypeScript build and fixture tests are separate from the Rust build and published product. Run `npm --prefix dev-tools/source-contracts ci --ignore-scripts`, then `npm --prefix dev-tools/source-contracts run verify`. Run extraction with `npm --prefix dev-tools/source-contracts run extract -- --source PATH --out PATH [--strict]`. Rust developer commands use Rust libraries; compiler research runs through this npm package.

Source-model generation also requires rustfmt. Package verification
uses small offline fixtures. `verify-generated` extracts the pinned upstream
declarations and checks committed Rust, fetching into the ignored cache when needed.
`generate` emits/checks the complete authored portfolio, `sample-equipment`
extracts raw equipment packets and `compare-equipment` compares Rust probe results.
`sample-items` recursively samples Item sources, including embedded/subitems;
the Rust `item_generation_probe` example compares typed slices against raw source
projections. These private experiment commands are documented in the package README and
[shared Item report](../research/shared-item-source-generation.md). Rust tests
live under atlas-foundry-model; its source parsers need no Node runtime.
`sample-predicates` extracts selected predicate-bearing field contexts, including
ChoiceSet constructor inputs. The Rust `predicate_generation_probe` compares
typed values and reports source conflicts with a nonzero exit. See the
[recursive generation report](../research/recursive-source-generation.md).
`sample-rules` samples root/embedded Item rules; `compare-rules` compiles scratch
schema/authored portfolios and checks raw-value fidelity in Rust. Rejections and
unmodeled keys remain counted and exit nonzero. Foundry runtime admission is a
separate evidence state; see the [authored-rule report](../research/authored-rule-source.md).

The small `dev-tools/source-contracts/source-pin.json` records upstream identity.
Private `npm --prefix dev-tools/source-contracts run generate -- --check` fetches
the pin when needed, extracts afresh into ignored `.cache/source-contracts`,
and checks committed Rust under `crates/atlas-foundry-model/src/source_model/generated`.
Omit `--check` to regenerate. `--source PATH` seeds the cache from matching local
source without modifying that checkout. CI runs `verify-generated` separately
from offline fixture tests. Ordinary Rust builds require neither Node nor Foundry.

`atlas-foundry-model` owns pure strict/admission APIs and typed snapshots. Ingest
owns files/provenance and calls shared admission once. Model serialization
includes explicit union tags. `parse_source_value` reads authored JSON, while
Serde Deserialize reads typed source values; never use it to admit source files.

Run the standalone corpus proof against the authenticated source pin:

```sh
cargo run -p atlas-foundry-model --example snapshot_corpus -- /path/to/pinned/pf2e
```

It asserts exact Rust equality after encoding/decoding every root and checks
pinned loading outcomes. It is contributor validation, not runtime loading or
an artifact migration. Codec/source policy changes follow ADR 0043.

The database-independent record enrichment handoff is
`atlas-ingest::enrich_loaded_source`. It consumes admitted models, retains source
bytes/provenance and diagnostics, and calls `atlas-record::source_record` for
owned identity, selected content, references and typed query/text views. Audience
and implicit check-DC policy are required inputs; no product default is implied.

Run its bounded corpus evidence example with the authenticated source pin and
that pin's locale:

```sh
cargo run -p atlas-ingest --example source_enrichment_probe -- /path/to/pinned/pf2e /path/to/pinned/pf2e/static/lang/en.json
```

The JSON report checks all retained typed snapshots before/after enrichment,
compares selected query values independently to authored JSON, checks visible
reference/interaction markers, and measures sizes and stage/warm timings. Raw
JSON comparison exists only in this developer proof. The probe explicitly uses
a public audience with GM-only implicit DCs; it does not publish CLI commands,
persist corpus receipts or validate browser behavior, search relevance or SQLite
hydration. See [ADR 0045](../architecture/decisions/0045-source-backed-record-enrichment.md)
and the [projection inventory](../research/source-record-query-projections.md).


`compare-portfolio` checks these maintained models against all five document
kinds and specific rules through the Rust `source_portfolio_probe` example.
It reports every rejection and value difference, and compares the pinned corpus
baseline. Known failures still return nonzero, even when the baseline matches;
this tooling does not decide production ingest rejection policy. See the package
README and [maintained portfolio report](../research/maintained-source-portfolio.md).

Search-profile operations persist requested configuration independently of the
active reference manifest: fresh `atlas setup` prepares BGE Base; omitted flags
preserve requested profiles, and explicit `--search-profile` flags add to them. Search/web omission selects a sole stored
profile, or Base when both are built. Pinned model files live under
`hf-models/<model-id>/<revision>/`; setup repairs selected invalid files through
verified temporary download and rename. Do not relocate this path policy into
vector identity or add a legacy-layout reader.

Use `--progress always --json >report.json 2>progress.log` when reviewing normal
build progress. Stages emit concise operation labels and structured counters/units;
the CLI renderer owns counts and elapsed/estimated time. Keep paths, revisions,
reuse statistics and internal terminology out of normal progress text; use
`ATLAS_LOG=info` for diagnostic fields. Check selected model names, passage versus
unique-passage counts and bounded stage updates. ETA is approximate, omitted until
there is a useful sample, and rounded to minutes for long-running stages. Include an unchanged
reuse-heavy tiny build and `--progress never`; JSON stdout must stay parseable.
Rendering/control-plane corrections do not justify a full-corpus reembedding.

### Embedding storage regression probe

Normal setup/build reclaims substantial free SQLite pages after publication. An
ordinary startup may migrate the schema but does not vacuum; run `atlas setup`
explicitly to reclaim space after a storage upgrade. A failed reclamation retains
complete reference data and can be retried without repeating inference.

For the schema-9 to schema-10 storage change, an optional corpus probe captures
one consistent disposable snapshot, verifies every vector and source attribution,
compares saved-state hashes and filtered distances, checks exact-input reuse and
runs deep validation. It never migrates its source. Use a repository development
corpus, with enough temporary space for the snapshot, normal migration backup and
SQLite staging; all copies are removed on completion:

```bash
ATLAS_STORAGE_PROBE_SOURCE=/absolute/path/to/development/workspace.sqlite \
  cargo test -p atlas-dev --test embedding_storage -- --ignored --nocapture
```

These local debug timings are diagnostic, not release performance guarantees.
Exact-distance KNN ties may return different witnesses after packing; comparison
checks the distance sequence rather than arbitrary tied IDs.
