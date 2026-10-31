# Setup and maintenance

Use this guide to update reference content, check setup problems, back up your
saved work, or remove downloaded files.

## Update reference data

Run setup after installing Atlas and after upgrading to a newer version:

```bash
atlas setup
```

Setup prepares the reference content supported by your installed Atlas version.
It keeps your saved lists and encounters and reuses search data where possible.
Restart any running `atlas web` command after upgrading Atlas to use the new version.

## Check a setup problem

Check what Atlas needs without downloading or changing anything:

```bash
atlas setup --check
```

Follow the reported action. If setup was interrupted, rerun `atlas setup` to
resume it. You may need to wait until an update finishes before using Atlas again.
Do not delete your workspace to resolve a setup error: it contains your saved work.

Older pre-release versions used separate reference and saved-state files. Those
formats are not converted automatically; check the reported error before creating
a new workspace.

## Run setup without downloads

To rerun setup using files already on your computer:

```bash
atlas setup --offline
```

This flag controls setup's downloads. Normal local searches use the data you have
already prepared. Offline setup cannot fetch a missing source or search model.

## Data locations

Atlas stores reference data, saved lists and encounters in `workspace.sqlite`.
These are the default locations:

| System | Workspace file | Downloaded source and model folder |
| --- | --- | --- |
| macOS | `~/Library/Application Support/pf2e-atlas/workspace.sqlite` | `~/Library/Caches/pf2e-atlas/` |
| Linux | `~/.local/share/pf2e-atlas/workspace.sqlite` | `~/.cache/pf2e-atlas/` |
| Windows | `%LOCALAPPDATA%\pf2e-atlas\workspace.sqlite` | `%LOCALAPPDATA%\pf2e-atlas\` |

`~` means your home folder. On Windows, paste `%LOCALAPPDATA%\pf2e-atlas` into
File Explorer's address bar. On Linux, `XDG_DATA_HOME` and `XDG_CACHE_HOME` replace
`~/.local/share` and `~/.cache` if you have configured them.

On Windows, downloads and the workspace share a parent folder. Use the
[cleanup commands](#remove-downloaded-files) to remove downloads while keeping
your workspace.

Downloaded source and models can be fetched again. Your workspace contains saved
work that cannot be recovered by downloading reference content again.

## Back up and restore your workspace

Before testing a nightly or switching to an older Atlas version:

1. Stop `atlas web` with Ctrl+C and wait for all other Atlas commands, including
   agent lookups and setup, to finish.
2. Find `workspace.sqlite` using the table above. If you use `--index`, use that
   file's name and location for the remaining steps.
3. Copy the database file and any accompanying files whose names begin with its
   filename into a separate backup folder. For the default workspace, these are
   `workspace.sqlite` and any files beginning with `workspace.sqlite`. Keep the
   files together and leave the originals in place. Record the output of
   `atlas --version` alongside the backup.

The backup contains reference data as well as your lists and encounters. You do
not need to copy downloaded source or models. Keep Atlas stopped until copying
finishes so the files describe the same point in time.

To restore, stop all Atlas commands again. Move the current database file and its
accompanying files into a separate folder, then copy the backed-up files into
their original location, using the same filenames. Start with the Atlas version recorded for that
backup. Restoring returns your saved work to the point when the backup was made.
Switching executables alone does not undo changes to your workspace.

## Remove downloaded files

First preview what would be removed:

```bash
atlas setup clean --all --check
```

To remove those downloaded source and model files:

```bash
atlas setup clean --all --yes
```

Use `--embeddings` instead of `--all` to select only model files, or
`--source-checkout` to select only the downloaded PF2e source. `--check` always
previews without removing anything. Removing both caches requires `--yes`.

Cleanup keeps your workspace, lists and encounters. Searching by meaning needs
the model files; run `atlas setup` to download them again when needed.

## Uninstall Atlas

Remove the `atlas` executable from the location printed by the installer. Your
workspace remains at the location above. Cache cleanup and uninstalling the
executable do not remove your saved work.

## Advanced: choose other locations

Use these options when you need a different workspace or cache directory:

- `--index /path/to/workspace.sqlite` selects a workspace for the command.
- `--embedding-cache-path /path/to/hf-models` selects the model cache.
- `--source /path/to/pf2e` selects the source checkout for setup. Atlas does not
  update that checkout; use the supported PF2e version or setup reports a mismatch.

`--path-mode global` is the default and uses the locations above. Contributors
can use `--path-mode repo` inside a repository checkout for `.atlas/workspace.sqlite`,
`.cache/pf2e/<supported-commit>` and `.cache/hf-models`. See
[development setup](development/setup-and-preview.md) before preparing a development
workspace. Setup remembers its source, language and selected models for later runs.
