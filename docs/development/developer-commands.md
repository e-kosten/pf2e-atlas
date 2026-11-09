# Developer commands

## Choose an entry point

| Your task | Entry point |
| --- | --- |
| Set up, search, or serve an Atlas workspace | `atlas` |
| Inspect source or artifacts, run an evaluation, or generate a recurring maintainer report | `atlas-dev` |
| Run a checkout workflow that coordinates builds, previews, or validation | `just` |
| Use a separate toolchain, such as the Foundry TypeScript compiler | The owning package under `dev-tools/` |
| Probe a crate during an experiment or provide a focused executable for an automated caller | A crate-local Cargo example |

Keep recurring Rust reports reachable through `atlas-dev --help`. A command can
use the owning crate without moving that crate's product logic into the CLI.

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

These commands use the same global/repo path policy, JSON envelope, and progress
controls as atlas. `source load` reads generated authored DTOs, keeping original
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
Source commands' `--manifest` overrides
the Foundry input manifest. Reports go to stdout; `--json` selects JSON.
Use Cargo's release profile for full-corpus processing and ingest performance
measurements, including builds that reuse every embedding. Source preparation,
tokenization and publication validation remain substantial CPU work without
inference; ordinary debug builds are intended for development checks.

```bash
cargo run --release -p atlas-dev -- source analyze --source vendor/pf2e --json
```

Setup and index build/check/validate remain operational product commands:

```bash
cargo run --release -p atlas-cli -- index build --no-embeddings --json
```

For timings, build the executable first and measure its invocation separately
from compilation. Keep fresh builds, unchanged updates and content-policy upgrades
as separate cases; capture `--progress always` alongside the JSON report. Use an
isolated workspace copy for experiments, retain verified embedding checkpoints,
and avoid concurrent builds or benchmarks that would distort the comparison.

Foundry compiler research runs through the separate private
[TypeScript package](../../dev-tools/source-contracts/README.md), with npm entry points.
Rust developer commands do not launch TypeScript tools. Neither developer
surface is included in published product bundles; atlas-dev is built locally.


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
audit or `data.proposals` for grade candidates. The command leaves the workspace
and catalog files alone. See [equipment family review](source-research.md#equipment-family-review-during-a-source-upgrade)
for the source checks that precede catalog changes.
