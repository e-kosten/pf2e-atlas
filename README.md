# PF2e Atlas

The supported Foundry PF2e reference release is **8.6.1**. Setup acquires its pinned source revision.

PF2e Atlas gives players, GMs, tool builders, and local coding agents a fast way to search and reference Pathfinder Second Edition content locally. It supports record lookup, ranked search, structured filters, name resolution, similar-record retrieval, graph context, and a local web workspace.

The `atlas` CLI indexes PF2E data from the [Foundry PF2E repository](https://github.com/foundryvtt/pf2e) and exposes tools for people and agents.

## What You Can Do

With `atlas`, you can:

- browse reference records in the local web app, maintain saved lists, and run encounters
- search PF2E records by text, kind, and structured filters
- resolve names (including by pre-remaster aliases)
- find records similar to a known seed record
- inspect nearby references around a known record
- roll dice with individual faces and kept/dropped results, without runtime setup
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

`atlas setup` downloads the supported PF2E source and embedding model, builds
local reference/search data, and verifies readiness. The first embedding build
can take a while; later updates reuse compatible embeddings and retain saved
lists and encounters.

Open the browser workspace:

```bash
atlas web
```

Use the address printed by the command to search, preview records, organize lists
and run encounters. Encounter controls include initiative, HP, conditions, spell
resources and dice; automation covers supported mechanics rather than every
Foundry rule.

### Search models and ranking

Search supports two model profiles: `bge-base` (BGE Base) and
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
available when storage or memory is limited. `atlas setup --no-embeddings` creates a new
workspace for lexical search and reference lookup without models. It preserves
configured models in an existing workspace. Use `atlas search QUERY --retrieval fts`
for text queries in a lexical-only workspace.

Without a profile flag, search and web use the sole stored profile, or BGE Base
when both are stored. Running setup again without profile flags preserves the
requested set for source updates. Explicit setup flags add profiles to that set.
Setup repairs selected pinned assets in revision-specific cache directories;
switching a built profile does not download assets or rebuild the artifact.

Hybrid search also offers two ranking strategies for comparison: `score` (the
default, shown as **Score-based** in the browser) and `rrf` (**Rank-based**).
Both use the same lexical projection and selected embedding model. Change the
browser ranking picker or run `atlas search "harrowkin" --ranking score` to compare;
switching does not rebuild search data. Ranking applies to hybrid text search,
not alphabetical browsing or Similar records.

### Platforms and versions

The installer installs only the `atlas` CLI. The persistent workspace lives in user application-data storage; downloaded source and models stay in cache locations managed by `atlas setup`.

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

### Nightly builds

Nightlies are prerelease builds of `main` for everyday testing. They are published
when the scheduled build finds a new commit and all checks pass. Stable releases
remain an explicit maintainer action; the default installer selects stable.

Install or update to the newest successful nightly on macOS/Linux:

```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/e-kosten/pf2e-atlas/releases/download/nightly/atlas-installer.sh \
  | sh -s -- --version nightly
```

On Windows, download the installer and select the nightly version:

```powershell
Invoke-WebRequest https://github.com/e-kosten/pf2e-atlas/releases/download/nightly/atlas-installer.ps1 -OutFile atlas-installer.ps1
./atlas-installer.ps1 -Version nightly
```

The installer prints the dated release it resolved. `atlas --version` includes
that build's UTC timestamp and source commit prefix. Each dated release retains
its binaries, so you can reinstall a specific nightly with `--version <tag>`
(or `-Version <tag>`). Repeat the nightly command to update; Atlas does not
update itself automatically. Add `--yes` / `-Yes` for unattended replacement.

To return to stable, run the installer with `--version latest` / `-Version latest`.
Nightly and stable installations share the normal data paths; switching binaries
does not restore older local data. Back up saved state before testing changes that
migrate it, and rerun `atlas setup` if the selected build requires a rebuilt index.


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

Human record reading supports summary, standard (default), and full detail.
Full expands supported embedded records such as spells and journal pages.
`record get SPELL_KEY --rank 5` previews a spell at a selected rank without
spending encounter resources. Embedded spells retain their local casting context.

`record get --json` returns the complete selected Atlas reference response and
conflicts with explicit `--detail`. `record source --json` exports the checked
Foundry DTO, including information outside Atlas's presentation. It includes typed
presence/union tags and does not require the original source clone.

Discover filters and follow references:

```bash
atlas filters fields
atlas filters fields --group movement
atlas filters fields --field defenses.hp.maximum
atlas filters values --field traits --kind spell
atlas search --kind creature --where 'movement.speeds.exists(s, s.type == "fly" && s.feet >= 30)'
atlas similar 'Dirge of Doom' --kind spell
atlas graph uses 'Frightened' --limit 5
```

Filters use supported CEL expressions with discoverable types, units and examples.
Use explicit filters for constraints such as kind, level, rarity and traits;
omitting query text browses matching records alphabetically. Constraints inside
one collection `exists` apply to the same entry. Similar finds semantic neighbors;
graph commands follow explicit references.

Roll dice without a workspace or embedding model:

```bash
atlas roll '1d20+12'
atlas roll '2d20kh1+12' # Keep the higher die
atlas roll '(2d6+4)*2' --json
```

Use `kl` to keep the lowest dice. Human output shows the total and arithmetic;
JSON retains individual dice and kept/dropped states. These rolls do not change
encounter state.

See [CLI examples](docs/cli-examples.md) for batch reads, embedded selections,
filter collections, search comparison and saved lists.

### Saved lists

Create a list with resolved record keys, then add entries as needed:

```sh
atlas lists create fey-options --name "Fey Options" \
  --record pathfinder-monster-core:lUBkzsSqMfQBczU1 \
  --record pathfinder-monster-core:Ehtm5k9iBYTvSUcZ
atlas lists show fey-options
atlas lists add fey-options --record pathfinder-monster-core:lUBkzsSqMfQBczU1
atlas lists note fey-options pathfinder-monster-core:lUBkzsSqMfQBczU1 \
  --text "A potential garden ally"
```

Creation rejects an existing list id; use `lists add` to extend it. Batches report
added, already present and failed inputs. Valid entries are saved together; any
failed input gives a nonzero exit status. If every input fails resolution, the
new list is empty. A storage failure rolls back the complete action.

Notes belong to an entry in one list and survive reference updates and list
export/import. Use `lists note <ID> <KEY> --clear` to clear one. In the web app,
the selected entry's note stays beneath the scrolling preview.

Install shell completion with `atlas completions zsh` (or `bash`, `fish`,
`powershell`, `elvish`). For example, in zsh:

```sh
source <(atlas completions zsh)
```

Supported live completion shells suggest saved-list ids and canonical record keys
from the selected workspace, including `--index` and `--path-mode` overrides.
Completion skips workspace upgrades and model loading; an unavailable workspace
returns no saved-value suggestions. Regenerate completion scripts after upgrading.

## Agent Skill

The CLI includes a first-party skill that teaches local coding agents how to use Atlas for PF2E lookup, search, filter discovery, and graph context.

With Atlas installed, use its bundled skill (matched to that CLI version) and
choose the agent target you use:

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

After upgrading Atlas, rerun the skill installer to update the installed copy.
A differing copy requires `--force`; review local edits before replacing it.
The skill is bundled with the executable, so install it from the Atlas version
you intend the agent to use.

You can also install the skill independently with Node.js and the
[skills CLI](https://github.com/vercel-labs/skills):

```bash
npx skills add e-kosten/pf2e-atlas --skill pf2e-atlas-cli
```

The interactive installer lets you select agents and installation scope. For a
global Codex installation:

```bash
npx skills add e-kosten/pf2e-atlas --skill pf2e-atlas-cli --agent codex --global
```

This installs only the skill, not Atlas or its data. The skill includes CLI
installation and setup guidance. The repository's `main` skill tracks the nightly
CLI; use the bundled installer for a release-matched copy. Use the installation
method you chose to update that copy; Atlas's installer does not manage the
skills CLI's symlinks or lockfiles.

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
atlas setup clean --embeddings --check
atlas setup clean --all --yes
```

Cleanup supports `--embeddings`, `--source-checkout`, and `--all`, with `--check` for a dry run. It removes external caches, never the persistent workspace or saved lists and encounters. Cleanup that selects every cache target requires `--yes` unless `--check` is used.

To uninstall the CLI, remove the `atlas` binary from the path printed by the installer. Your workspace remains in application-data storage.

## Advanced Data Locations

The default setup stores the durable workspace in platform application-data storage and downloaded source/models in user cache storage. The installed CLI can be used from any directory after setup.

- `--path-mode global` uses `pf2e-atlas/workspace.sqlite` under platform application data, with external caches under the platform user cache directory.
- `--path-mode repo` is a contributor mode that uses paths inside this repository:
  - pinned source cache: `.cache/pf2e/<supported-commit>`
  - embedding model cache: `.cache/hf-models`
  - persistent workspace: `.atlas/workspace.sqlite`

Default reference data and generated models use the same immutable PF2e revision
(currently 8.6.1). Setup fetches that revision rather than following upstream's
latest branch. Supplied source directories are never pulled or checked out by
Atlas; incompatible PF2e system versions produce an expected/detected version error.

You can select workspace/cache paths for a command and override the source used by setup:

- `--source /path/to/pf2e`
- `--output /path/to/workspace.sqlite`
- `--index /path/to/workspace.sqlite`
- `--embedding-cache-path /path/to/hf-models`

Setup updates reference data inside the existing workspace, preserving lists and
encounters. Source, manifest, locale and requested profiles persist independently
of successful builds. Schema upgrades use migrations; supported new filter
projections are rebuilt from stored source DTOs. Changed preparation or model
inputs require corresponding work, while exact compatible vectors are reused
from durable checkpoints. Interrupted work can resume without discarding saved
state or already committed inference batches. Maintenance temporarily prevents
normal access during publication or upgrades. The older separate-index/state
pre-release formats require a fresh workspace;
they are not converted automatically. Do not delete a workspace to resolve a
readiness error; inspect `atlas setup --check` first.

## Automation And JSON

Most commands support `--json`. Successful payloads are under `data`, and
command-wide failures are under `error`. `record get` and `record resolve` return
ordered `data.results` with per-input status and successful values or errors;
the outer status may be `partial`. Preserve successful siblings and inspect both
the payload and exit code. See [CLI examples](docs/cli-examples.md#readiness-and-json)
for the batch and exit-code contract.

Artifact validation against an invalid artifact is still a successful command invocation: it returns `status: "ok"` with `data.valid: false` and exits with code `3`.

Example:

```bash
atlas index validate --json
```

## Install From Source

For source installation, install Rust 1.95 or newer and Node.js 22 (the CI version).
Build the frontend once, then install the CLI (which embeds that frontend):

```bash
npm --prefix web/atlas-ui ci
npm --prefix web/atlas-ui run build
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
