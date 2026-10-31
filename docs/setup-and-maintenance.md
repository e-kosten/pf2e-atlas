# Setup and maintenance

The default [quick start](../README.md#quick-start) needs no path or model flags.
Use this guide for readiness problems, updates, cache cleanup and custom data locations.

## Update reference data

After [updating the CLI](installation.md), run:

```bash
atlas setup
```

Setup acquires the pinned Foundry PF2e source supported by that Atlas build.
It preserves saved lists and encounters and reuses compatible embeddings.
Restart any running `atlas web` process after a CLI upgrade to use the new version.

## Readiness and offline setup

Check setup status without changing local data:

```bash
atlas setup --check
```

Use already-downloaded data without network access:

```bash
atlas setup --offline
```

## Cache cleanup and uninstall

Preview cleanup targets before removing caches:

```bash
atlas setup clean --embeddings --check
atlas setup clean --all --yes
```

Cleanup supports `--embeddings`, `--source-checkout`, and `--all`, with `--check`
for a dry run. It removes external caches, never the workspace or saved lists and
encounters. Selecting every cache target requires `--yes` unless using `--check`.
Later semantic text searches need the model cache restored with `atlas setup`.

To uninstall the CLI, remove the `atlas` binary from the path printed by the
installer. Your workspace remains in application-data storage.

## Data locations

The default setup stores the workspace in platform application-data storage and
source/models in user cache storage. Atlas can be used from any directory after setup.

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
- `--index /path/to/workspace.sqlite`
- `--embedding-cache-path /path/to/hf-models`

Setup remembers your source, locale and requested models for later runs. It
resumes interrupted preparation and reuses compatible completed embeddings.
Publication and upgrades may temporarily prevent normal access.

The old separate-index/state pre-release formats require a fresh workspace and
are not converted automatically. Inspect `atlas setup --check` before responding
to a readiness error; deleting your workspace would discard saved state.
