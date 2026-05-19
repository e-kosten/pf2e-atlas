# PF2e Atlas

PF2e Atlas is a local Pathfinder 2E search and reference runtime built from the [Foundry PF2E repository](https://github.com/foundryvtt/pf2e). The `atlas` CLI installs and repairs local data, builds the SQLite artifact, validates readiness, and exposes record lookup, strict name resolution, graph context, filter discovery, and ranked search.

## Capabilities

With `atlas`, you can:

- fetch and maintain a local Foundry PF2E source checkout
- build and validate a local SQLite artifact for PF2E records
- look up detailed PF2E records by canonical key
- resolve names and verified aliases to canonical records
- search and list records with structured filters
- inspect one-hop graph context around known records
- install the first-party PF2e Atlas skill for local coding agents

## Quick Start

Install Rust and Cargo with [rustup](https://rustup.rs/) or your platform package manager.

Install the local CLI from this clone:

```bash
cargo install --path crates/atlas-cli --locked
```

Run the standard first-time setup:

```bash
atlas setup
```

`atlas setup` resolves the default paths, fetches or updates the Foundry PF2E source checkout, prepares the configured embedding model cache, builds or repairs the local SQLite artifact, and validates the result. Embeddings are required by default because semantic search depends on them.

For a faster record-only setup, use:

```bash
atlas setup --no-embeddings
```

That produces a base artifact suitable for `record get`, `record resolve`, and filter-only listing, but not semantic search.

## Try It

After setup, commands use the resolved default artifact path automatically:

```bash
atlas record get actionspf2e:1kGNdIIhuglAjIp9
atlas record get equipment-srd:s1vB3HdXjMigYAnY
atlas record resolve "Treat Wounds" --pack-name actionspf2e
atlas search "low level healing spell" --family spell --limit 5
atlas filters fields
atlas graph get actionspf2e:1kGNdIIhuglAjIp9
atlas index validate
```

Useful setup and validation variants:

```bash
atlas setup --check
atlas setup --offline
atlas setup clean --artifact
atlas setup clean --all --yes
atlas index validate --no-embeddings
atlas index validate --embeddings-only
```

`atlas setup --check` reports readiness and planned actions without writing local runtime files. `atlas setup --offline` prevents network-backed source updates and embedding model preparation. `atlas setup clean` removes selected runtime data without uninstalling the CLI; use `--artifact`, `--embeddings`, `--source-checkout`, or `--all`, with `--check` for a dry run. Cleanup that selects every target requires `--yes` unless `--check` is used.

## Agent Skill

The CLI includes a first-party PF2e Atlas skill package for local coding agents. Inspect install readiness with:

```bash
atlas agent skills doctor --json
```

Install into the current workspace with:

```bash
atlas agent skills install --target agents --scope workspace --yes --json
```

Supported targets are `agents`, `claude`, `codex`, `copilot`, `gemini`, and `kiro`. Installed Atlas-managed skills include an `.atlas-skill.json` manifest with the package content hash used by doctor and install.

## Paths And Data

`atlas` uses the runtime path resolver:

- `--path-mode global` uses platform user cache paths under `pf2e-atlas`, so setup and later commands use the same data from any working directory.
- `--path-mode repo` is an explicit contributor mode that requires running inside this repository and uses repo-local paths:
  - source: `vendor/pf2e`
  - embedding model cache: `.cache/hf-models`
  - SQLite artifact: `.cache/pf2e-rust-index.sqlite`

Direct command flags override resolved paths for that command, such as:

- `--source /path/to/pf2e`
- `--output /path/to/pf2e-rust-index.sqlite`
- `--index /path/to/pf2e-rust-index.sqlite`
- `--embedding-cache-path /path/to/hf-models`

The local SQLite artifact is reusable. Setup rebuilds or repairs it when validation fails, when the source signature is stale, when embedding metadata does not match the selected model, or when the embedding-unit policy changes.

## JSON Output

Most commands support `--json`. Successful command payloads are under `data`, and command failures are under `error`.

Artifact validation against an invalid artifact is still a successful command invocation: it returns `status: "ok"` with `data.valid: false` and exits with code `3`.

Example:

```bash
atlas index validate --json
```

## Manual Index Commands

Standard users should run `atlas setup`. Manual index commands remain available for development and diagnostics:

```bash
cargo run -p atlas-cli -- index analyze --json
cargo run -p atlas-cli -- index build --no-embeddings --json
```

Use Cargo's release profile for ingest or search performance measurements:

```bash
cargo run --release -p atlas-cli -- index analyze --source vendor/pf2e --json
```

## Further Reading

- [CONTRIBUTING.md](./CONTRIBUTING.md) for contributor workflow, validation commands, and repo layout
- [docs/architecture](./docs/architecture/overview.md) for architecture, artifact, and crate ownership docs

## Contact

For questions about PF2e Atlas, use the [GitHub issue tracker](https://github.com/e-kosten/pf2e-atlas/issues).

## Paizo Community Use Notice

PF2e Atlas uses trademarks and/or copyrights owned by Paizo Inc., used under Paizo's Community Use Policy (paizo.com/licenses/communityuse). We are expressly prohibited from charging you to use or access this content. PF2e Atlas is not published, endorsed, or specifically approved by Paizo. For more information about Paizo Inc. and Paizo products, visit paizo.com.
