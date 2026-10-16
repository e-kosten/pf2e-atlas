# PF2e Atlas

PF2e Atlas gives players, GMs, tool builders, and local coding agents a fast way to search and reference Pathfinder Second Edition content locally. It supports record lookup, ranked search, structured filters, name resolution, similar-record retrieval, graph context, and a local web workspace.

The `atlas` CLI indexes PF2E data from the [Foundry PF2E repository](https://github.com/foundryvtt/pf2e) and exposes tools for people and agents.

## What You Can Do

With `atlas`, you can:

- browse reference records in the local web app, maintain saved lists, and run encounters
- search PF2E records by text, kind, and structured filters
- resolve names (including by pre-remaster aliases)
- find records similar to a known seed record
- inspect nearby references around a known record
- install a PF2e Atlas skill that guides local agents through lookup and discovery

## Quick Start

Install the latest release:

```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/e-kosten/pf2e-atlas/releases/latest/download/atlas-installer.sh | sh
```

On Windows, use PowerShell:

```powershell
irm https://github.com/e-kosten/pf2e-atlas/releases/latest/download/atlas-installer.ps1 | iex
```

Run the standard first-time setup:

```bash
atlas setup
```

`atlas setup` fetches or updates the PF2E source data, prepares semantic search support, builds (and repairs) local search data, and verifies that Atlas is ready to use.

Search supports two baseline model profiles: `bge-base` (BGE Base) and
`embeddinggemma` (EmbeddingGemma). Fresh setup uses BGE Base. Prepare either profile,
or repeat the flag to keep both in one artifact:

```bash
atlas setup --search-profile bge-base --search-profile embeddinggemma
atlas web --search-profile bge-base
atlas search "forest guardian" --search-profile embeddinggemma
atlas similar "Treat Wounds" --search-profile bge-base
```

An explicit profile must be stored in the artifact; text semantic search also
requires its verified model cache. Similar uses stored vectors without loading
model weights. Each request uses one profile. A single-profile setup remains
available when storage or memory is limited. `atlas setup --no-embeddings` keeps
lexical search and reference lookup available without models.

Without a profile flag, search and web use the sole stored profile, or BGE Base
when both are stored. Running setup again without profile flags preserves the
stored set for source updates. Explicit setup flags choose the complete desired
set. Setup repairs selected pinned assets in revision-specific cache directories;
switching a built profile does not download assets or rebuild the artifact.

The installer installs only the `atlas` CLI. Runtime data stays in user cache locations and is managed by `atlas setup`.

Release binaries are provided for Linux x64, Linux ARM64, macOS Apple Silicon,
and Windows x64. macOS Intel and Windows ARM64 are not published as release
binaries yet.

Confirm the installed version:

```bash
atlas --version
```

To install a pinned version, use that release's installer URL:

```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/e-kosten/pf2e-atlas/releases/download/v0.1.0/atlas-installer.sh | sh
```

Release candidates are installed only from pinned URLs or with `ATLAS_VERSION`.

If you do not want to pipe an installer into a shell, download the cargo-dist archive for your OS from the [GitHub releases page](https://github.com/e-kosten/pf2e-atlas/releases), verify it against `SHA256SUMS`, extract `atlas`, put it on `PATH`, then run `atlas setup`.

Early release binaries are unsigned. macOS or Windows may show an operating-system warning until signing and notarization are added in a later release.

## Try It

Search for something by meaning:

```bash
atlas search "low level healing spell" --kind spell --limit 5
```

Resolve a record by name:

```bash
atlas record resolve "Treat Wounds" --pack-name actionspf2e
```

Fetch records by keys (generally retrieved from other results and queries):

```bash
atlas record get actionspf2e:1kGNdIIhuglAjIp9
atlas record get equipment-srd:s1vB3HdXjMigYAnY
atlas record get equipment-srd:s1vB3HdXjMigYAnY --detail full
atlas record get equipment-srd:s1vB3HdXjMigYAnY --json
atlas record source equipment-srd:s1vB3HdXjMigYAnY --json
```

Human record reading supports summary, standard (the default), and full detail.
Prepared HTML uses terminal emphasis when stdout is an interactive terminal.
Pipes, files, `TERM=dumb` and nonempty `NO_COLOR` use plain text formatting.
`record get --json` returns the complete selected Atlas reference response;
explicit `--detail` and `--json` conflict. `record source --json` instead exports
the complete checked Foundry DTO from the artifact, including embedded documents
and fields outside Atlas's presentation. Its presence/union tags describe the
checked model; it is not original Foundry JSON and needs no source clone.

Explore available filters and nearby record context:

```bash
atlas filters fields
atlas filters fields --text 'maximum hp'
atlas filters fields --field actor.hp.maximum
atlas filters values --field actor.hp.maximum --kind creature
atlas similar "Dirge of Doom" --kind spell
atlas graph links actionspf2e:1kGNdIIhuglAjIp9
```

`filters fields` shows a compact field index. `--field` shows a complete
descriptor, and `--text` finds labels, IDs, scopes and source families. JSON keeps
complete descriptors, narrowed only when requested. `filters values` reports
numeric ranges or collection availability for those field types; categorical
fields provide paged values alongside availability counts.

## Agent Skill

The CLI includes a first-party skill that teaches local coding agents how to use Atlas for PF2E lookup, search, filter discovery, and graph context.

Run the interactive installer and choose the agent target you use:

```bash
atlas agent skills install
```

For scripts or setup automation, provide the target and scope directly:

```bash
atlas agent skills install --target agents --scope workspace --yes --json
```

Supported targets are `agents`, `claude`, `codex`, `copilot`, `gemini`, and `kiro`.

To inspect existing installs or diagnose a changed skill directory, use:

```bash
atlas agent skills doctor
```

## Maintenance

Check setup status without changing local data:

```bash
atlas setup --check
```

Use already-downloaded data without network access:

```bash
atlas setup --offline
```

Remove selected local data:

```bash
atlas setup clean --artifact
atlas setup clean --all --yes
```

Cleanup supports `--artifact`, `--embeddings`, `--source-checkout`, and `--all`, with `--check` for a dry run. Cleanup that selects every target requires `--yes` unless `--check` is used.

To remove Atlas completely, first remove runtime data:

```bash
atlas setup clean --all --yes
```

Then remove the `atlas` binary from the path printed by the installer.

## Advanced Data Locations

The default setup stores Atlas data in platform user cache paths, so the installed CLI can be used from any directory after setup.

- `--path-mode global` uses the default user cache location under `pf2e-atlas`.
- `--path-mode repo` is a contributor mode that uses paths inside this repository:
  - source: `vendor/pf2e`
  - embedding model cache: `.cache/hf-models`
  - SQLite artifact: `.cache/pf2e-index.sqlite`

You can override locations for a single command:

- `--source /path/to/pf2e`
- `--output /path/to/pf2e-index.sqlite`
- `--index /path/to/pf2e-index.sqlite`
- `--embedding-cache-path /path/to/hf-models`

Setup rebuilds or repairs local search data when validation fails, source data
changes, or requested profile coverage/contracts change. Compatible unchanged
profile vectors can be reused independently. Changing a model or passage recipe
does not invalidate unchanged shared authored-content preparation.

## Automation And JSON

Most commands support `--json`. Successful command payloads are under `data`, and command failures are under `error`.

Artifact validation against an invalid artifact is still a successful command invocation: it returns `status: "ok"` with `data.valid: false` and exits with code `3`.

Example:

```bash
atlas index validate --json
```

## Install From Source

For source installation from a local checkout, install Rust 1.95 or newer and run:

```bash
cargo install --path crates/atlas-cli --locked
```

See [CONTRIBUTING.md](./CONTRIBUTING.md) for source development and diagnostic commands.

## Further Reading

- [docs/getting-started.md](./docs/getting-started.md) for first-run setup and core lookup workflows
- [docs/cli-examples.md](./docs/cli-examples.md) for task-oriented CLI recipes
- [CONTRIBUTING.md](./CONTRIBUTING.md) for contributor workflow, validation commands, and repo layout
- [docs/architecture](./docs/architecture/overview.md) for architecture, artifact, and crate ownership docs

## Contact

For questions about PF2e Atlas, use the [GitHub issue tracker](https://github.com/e-kosten/pf2e-atlas/issues).

## Paizo Community Use Notice

PF2e Atlas uses trademarks and/or copyrights owned by Paizo Inc., used under Paizo's Community Use Policy (paizo.com/licenses/communityuse). We are expressly prohibited from charging you to use or access this content. PF2e Atlas is not published, endorsed, or specifically approved by Paizo. For more information about Paizo Inc. and Paizo products, visit paizo.com.
