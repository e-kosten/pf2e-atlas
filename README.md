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
- roll dice with individual faces and kept/dropped results, without runtime setup
- install a PF2e Atlas skill that guides local agents through lookup and discovery

## Quick Start

General dice rolls need no index or embedding model:

```bash
atlas roll '1d20+12'
atlas roll '2d20kh1+12' # Keep the higher die
atlas roll '(2d6+4)*2' --json
```

Use `kl` to keep the lowest dice. Quote expressions containing spaces or shell
operators; use `atlas roll -- '-1d6+10'` for a leading minus sign. Human output
shows the total and arithmetic breakdown; JSON retains every die, its sides and
kept state, and intermediate totals. These rolls do not change encounter state.

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
requested set for source updates. Explicit setup flags add profiles to that set. Setup repairs selected pinned assets in revision-specific cache directories;
switching a built profile does not download assets or rebuild the artifact.

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

With `--json`, `record get` and `record resolve` return one `data.results` entry
per input, in order, preserving duplicates. Each entry has `input` and `status`,
plus `value` on success or `error` on failure. The outer status is `ok`, `partial`,
or `error`, so successful siblings remain usable when another input fails.
Runtime failures stop further work and mark remaining inputs `unprocessed`.
Exit codes distinguish success (0), misses/ambiguity (1), invalid input (2),
and runtime failures (3); mixed failures use the highest code. Command-wide
failures, such as an unavailable workspace, use a top-level `error` instead.

Human record reading supports summary, standard (the default), and full detail.
Use `record get <spell-key> --rank 5` to preview supported spell mechanics at a
selected rank; `--json` carries the same effective facts. Owned spells retain
their local customizations and use their casting context by default. Rank
preview never spends encounter resources.
Prepared HTML uses terminal emphasis when stdout is an interactive terminal.
Pipes, files, `TERM=dumb` and nonempty `NO_COLOR` use plain text formatting.
Each successful `record get --json` value has `identity` for what you are reading,
`selected` for its exact navigation address, and `content` for prepared text
`fields`, structured `body`, classification and embedded navigation. An embedded
selection also includes `parent`, identifying its containing record. Root responses
omit `parent` and serialize identity once. Rank previews report effective facts
in the selected identity and content.

`record get --json` returns the complete selected Atlas reference response;
explicit `--detail` and `--json` conflict. `record source --json` instead exports
the complete checked Foundry DTO from the artifact, including embedded documents
and fields outside Atlas's presentation. Its presence/union tags describe the
checked model; it is not original Foundry JSON and needs no source clone.

Explore available filters and nearby record context:

```bash
atlas filters fields
atlas filters fields --text 'maximum hp'
atlas filters fields --field defenses.hp.maximum
atlas filters values --field defenses.hp.maximum --kind creature
atlas similar "Dirge of Doom" --kind spell
atlas graph links actionspf2e:1kGNdIIhuglAjIp9
```

`filters fields` lists the same product groups used by the UI. Use `--group movement`
to browse Movement and senses, `--field movement.speeds` to inspect a collection,
or `--text speed` to search across groups. `--all` lists the full flat catalog.
JSON defaults to the shared grouped editor metadata; `--group` narrows those
groups. `--field`, `--text` and `--all` return catalog descriptors.
`filters values` reports
numeric ranges or collection availability for those field types; categorical
fields provide paged values alongside availability counts.

`filters fields --field ID` describes allowed choices. `filters values --field ID`
counts records under the supplied filters: closed fields retain zero-match
choices, while open fields enumerate observed values. Its `--text` searches values,
not record descriptions. Use `--offset` and `--limit` to page.

Value counts count each parent record once per value and respect remaster
preference. Source-availability statistics are before remaster suppression;
collection occurrences count entries and can exceed the number of records.
Unsupported discovery contexts return an error, not zero counts.

CEL expresses collection constraints directly:

```bash
# Creatures that speak Common
atlas search --kind creature --where '"common" in languages'
# Flying speed of at least 30 feet
atlas search --kind creature --where 'movement.speeds.exists(s, s.type == "fly" && s.feet >= 30)'
# Fire resistance of at least 10
atlas search --kind creature --where 'defenses.resistances.exists(r, r.type == "fire" && r.value >= 10)'
# At least one embedded fire spell of rank 3 or higher
atlas search --kind creature --where 'embedded.spells.exists(i, "fire" in i.traits && i.spell.rank >= 3)'
```

Conditions inside one `exists(...)` must match the **same entry**. Separate
`exists(...)` expressions can match different entries—for example, flight on one
movement entry and a high speed on another. Scoped discovery IDs such as
`movement.speeds[].type` identify catalog fields; use `s.type` inside CEL's `exists`.
Inspect `filters fields --field movement.speeds` to list the collection's entry fields,
then `filters fields --field 'movement.speeds[].type'` for that field's choices and
examples. `movement.speeds[]` alone is not a field ID. JSON `--field` output remains
the exact requested descriptor; use `--text movement.speeds --json` to discover the
collection and its member descriptors together.
CLI value discovery retains the entire supplied filter rather than replacing an
edited condition as a UI picker does. Its scoped value counts can match a different
entry from an existing collection condition; use `search --where` to test the
complete same-entry query.

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
atlas setup clean --embeddings --check
atlas setup clean --all --yes
```

Cleanup supports `--embeddings`, `--source-checkout`, and `--all`, with `--check` for a dry run. It removes external caches, never the persistent workspace or saved lists and encounters. Cleanup that selects every cache target requires `--yes` unless `--check` is used.

To remove downloaded source and model caches:

```bash
atlas setup clean --all --yes
```

To uninstall the CLI, remove the `atlas` binary from the path printed by the installer. Your workspace remains in application-data storage.

## Advanced Data Locations

The default setup stores the durable workspace in platform application-data storage and downloaded source/models in user cache storage. The installed CLI can be used from any directory after setup.

- `--path-mode global` uses `pf2e-atlas/workspace.sqlite` under platform application data, with external caches under the platform user cache directory.
- `--path-mode repo` is a contributor mode that uses paths inside this repository:
  - pinned source cache: `.cache/pf2e/<supported-commit>`
  - embedding model cache: `.cache/hf-models`
  - persistent workspace: `.atlas/workspace.sqlite`

Default reference data and generated models use the same immutable PF2e revision
(currently 6.12.4). Setup fetches that revision rather than following upstream's
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
normal access during publication or upgrades. Existing pre-release database
formats require an explicit fresh workspace; they are not converted automatically.

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
