# Developer commands

## Choose an entry point

Run these commands from the repository root unless the linked procedure says
otherwise. Use `just --list` for checkout workflows and
`cargo run --locked -p atlas-dev -- --help` for Rust developer commands.

| Your task | Start here | Inputs and outputs |
| --- | --- | --- |
| Prepare or check an Atlas workspace | `atlas setup --check --path-mode repo` | Reports readiness for the checkout workspace. Plain setup can acquire assets and update the workspace. See [product setup](../getting-started.md). |
| Inspect source, an index, tags, or equipment families | `cargo run --locked -p atlas-dev -- --help` | Reads the selected inputs and reports to stdout. See [Rust diagnostics](#rust-diagnostics) and [equipment families](#review-equipment-families). |
| Evaluate Similar retrieval with frozen inputs | `cargo run --locked -p atlas-dev -- similarity evaluate --help` | Reads a manifest, corpus, and freeze receipt. Select a scenario and the matching workspace for stored-vector phases. Writes reports to `--output-dir`. |
| Seed or inspect a development workspace | `just dev-workspace-init` and `just dev-status --json` | Init copies the shared seed only when the private workspace is missing. Status reports the selected backend and workspace readiness. See [development baselines](setup-and-preview.md#reusable-development-baseline). |
| Build or refresh a human preview | `just dev-preview` or `just dev-refresh` | Builds checkout-local outputs and starts or refreshes the server against private data. See [managed previews](setup-and-preview.md#managed-candidate-previews). |
| Plan or run local validation | `just verify-changed --range origin/main...HEAD --full --plan` | Explains selected checks. Omit `--plan` to run them and retain evidence in `.atlas/validation`. See [validation](validation.md). |
| Check generated HTTP bindings or run a browser fixture | [OpenAPI freshness](validation.md#openapi-freshness) or [browser checks](validation.md#local-browser-checks-and-the-ci-matrix) | The OpenAPI check compares regenerated files with tracked output. Browser checks use a disposable SQLite workspace. |
| Research Foundry declarations or check release tooling | [Source contracts](../../dev-tools/source-contracts/README.md) or [release tools](../../dev-tools/release/README.md) | Each private npm package owns its inputs, generated files, and checks. |

When adding a tool, use `atlas` for product operations such as workspace setup,
index repair, and per-record variant lookup. Use `atlas-dev` for recurring Rust
maintainer reports. Keep shared behavior in its owning crate. Use Just recipes
for checkout orchestration and private packages for separate toolchains. Keep a
narrow crate probe or an executable called by a specific harness in that crate's
Cargo examples. OpenAPI export and the browser fixture are invoked by their
respective generator and test harness. See [ADR 0033](../architecture/decisions/0033-developer-command-surfaces.md).

## Rust diagnostics

Build and run developer diagnostics independently of the product CLI, web UI
bundle, and Node:

```bash
cargo run -p atlas-dev -- --help
cargo run -p atlas-dev -- source load --source vendor/pf2e --json
cargo run -p atlas-dev -- source analyze --source vendor/pf2e --json
cargo run -p atlas-dev -- source audit-paths --source vendor/pf2e --record-type npc --json
cargo run -p atlas-dev -- index inspect --index .atlas/workspace.sqlite --json
cargo run -p atlas-dev -- index record actionspf2e:1kGNdIIhuglAjIp9 --index .atlas/workspace.sqlite --json
cargo test -p atlas-dev -p atlas-cli-support
```

Source and index diagnostics use the same path policy as `atlas`. The default
is global paths. Use `--path-mode repo` for checkout defaults, or pass
`--source` or `--index` explicitly. SQLite diagnostics read workspace data
without updating it. Opening a workspace can create a lock sidecar, and a
missing path can cause its parent directory to be created.

Reports go to stdout. Diagnostic `--json` uses the standard envelope, and
`--progress always` sends progress to stderr. Baseline `--json` reports and
`baseline capabilities` use raw JSON. Baseline web commands stream server output.

`source load` reads generated authored DTOs, keeping original
bytes, raw values, provenance and every admission
diagnostic. Its report counts modeled, partial, raw-only and quarantined outcomes
separately. It applies no source defaults, coercions, metric extraction or
embedding work. Embedded children and authored links remain in their source DTOs;
generic rule retention does not imply specific-rule interpretation.
Partial modeled documents produce exit 0; raw-only roots, quarantine, unavailable
packs or empty input produce exit 1 with the report on stdout. Root/manifest
errors produce exit 2. JSON `status: ok` means a report was produced; check the
counts and exit code for completeness. The loading API retains the full source
corpus in memory, with quarantine bytes retained only for successful file reads.
Analysis reports typed source selection and admission without writing SQLite;
path auditing reports observed paths and values. Index inspection reads an
existing artifact without changing it. `index record` decodes its checked source
snapshot and developer provenance; `--original --source vendor/pf2e` explicitly
reads the original clone file only after checking its recorded path and hash.
Source commands' `--manifest` overrides the Foundry input manifest.
Use Cargo's release profile for full-corpus processing and ingest performance
measurements, including builds that reuse every embedding. Source preparation,
tokenization and publication validation remain substantial CPU work without
inference; ordinary debug builds are intended for development checks.

```bash
cargo run --release -p atlas-dev -- source analyze --source vendor/pf2e --json
```

## Measure an index build

Use the product's `atlas index build` to measure publication. It initializes or
updates the SQLite path supplied as `--output`. Set `source_root` to the local
[pinned source checkout](source-research.md#pinned-pf2e-data), then build the
executable separately from the measured invocation. This example writes a
records-only artifact to a fresh temporary directory.

```bash
cargo build --locked --release -p atlas-cli --no-default-features
build_dir="$(mktemp -d)" &&
  target/release/atlas index build --path-mode repo \
    --source "${source_root:?Set source_root to the local pinned PF2e checkout}" \
    --output "$build_dir/workspace.sqlite" --no-embeddings --progress always --json
```

`--no-embeddings` leaves requested profiles in an existing workspace intact,
so use a fresh output to measure a records-only build. Keep fresh builds,
unchanged updates, and content-policy upgrades separate. Use an isolated copy
for update measurements. `atlas index check` tests readiness and
`atlas index validate` checks the artifact in depth. Both use `--index` for
the existing workspace path.


## Validate authored tags

Run `atlas-dev tags validate` from the repository root to validate the catalog,
assignments and ontology suggestions under `data/tags`. Use `--path` for a
different corpus root and `--json` for the validation report.


## Review equipment families

From the Atlas checkout, run the family audit against a private development
workspace. Use `--path-mode repo` for the checkout default or pass `--index`
for a separate preview copy.

```sh
cargo run --locked -p atlas-dev -- families audit --path-mode repo
cargo run --locked -p atlas-dev -- families audit --path-mode repo --propose-grades --json > proposals.json
```

The audit reads the entire equipment pack in a single workspace snapshot.
Add `--json` to get the standard JSON envelope with `data.records` for the
audit or `data.proposals` for grade candidates. The command preserves workspace data
and catalog files. See [equipment family review](source-research.md#equipment-family-review-during-a-source-upgrade)
for the source checks that precede catalog changes.
