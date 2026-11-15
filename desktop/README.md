# Develop the macOS desktop app

Build a local app from the checkout on Apple Silicon. You need the Xcode
command-line tools, the pinned [Rust toolchain](../rust-toolchain.toml) and Node 26.

```sh
npm --prefix web/atlas-ui ci
npm --prefix desktop ci
sh desktop/bundle-macos.sh
```

For the published DMG, see [macOS desktop installation](../docs/installation.md#macos-desktop-app).

Open **PF2e Atlas.app** in `target/debug/bundle/macos`. Keep the bundle together
with its CLI in `Contents/MacOS/atlas`. This build is for local testing.

## Use the app

Choose **Set up Atlas** when the workspace is empty. The app downloads the
pinned reference data and opens the workspace in the same window as soon as
the keyword index is ready. You can browse and search while Atlas prepares
semantic search. Saves are paused across Atlas sessions during setup and resume
when setup ends. Open **Setup and Updates** in the toolbar to see progress or cancel.
If you cancel after keyword search is ready, use **Resume semantic setup** to
continue later. If precomputed embeddings are unavailable, choose
**Retry download** or **Build embeddings locally**. The local build can take an
hour or more. When semantic search becomes ready, the next submitted search uses
it by default. Use **Rerun this search** to apply it to the current keyword results.
If the view fails to open, choose **Retry opening Atlas** to
use the prepared workspace. If Atlas asks you to upgrade
a workspace, close any other Atlas sessions using that workspace and choose
**Upgrade workspace**.

The toolbar and **PF2e Atlas → Setup and Updates** open the same drawer.
Each Atlas build pins a Foundry PF2e revision. After installing a build with
a different pin, confirm **Apply bundled reference data** to prepare that
build's reference data. Use **Check and repair reference data** to check the
current pin and prepare any missing assets. Atlas keeps your selected record
and drafts open. If the reference data changed while Search was open, use
**Refresh results** when prompted to read from the new version. For a custom
source, use the Atlas CLI to maintain its reference data.

Use **File → New Tab** (⌘T) or **File → New Window** (⌘N) to open a fresh
Search view. Command-click, middle-click, or choose **Open Link in New Tab**
from an Atlas link’s context menu to open its destination in a background tab.
All windows and tabs share the local workspace and service with a compatible
`atlas web` session. Choose **Quit PF2e Atlas** to release the app’s connection
to the service.

Use **View → Zoom In** (⌘+), **Zoom Out** (⌘−), or **Actual Size** (⌘0) to
scale the entire view. Atlas saves one zoom level for all windows and tabs.

Use the link icon in an entity header, or choose **Edit → Copy Atlas Link**
for the page's main entity. On a list page, the Edit menu copies the list link.
Use the reader's icon to copy the record link. The menu item becomes available
when the entity loads.

Paste into a rich-text editor for a linked entity name or into a plain-text
field for the `pf2e-atlas://v1/…` URL. Atlas links continue to work if the local
service moves to a different port. Saved list and encounter links use stable
keys and require that entity in the active workspace. In the browser, the same
icon copies the entity's web URL.

Opening an Atlas link from another app or with `atlas open '<link>'` selects an
existing matching tab, opens a foreground tab in the most recently active Atlas
window, or opens a window directly to the destination.

For shell or agent use, **PF2e Atlas → Terminal and Agent Access** can
link the bundled command at `~/.local/bin/atlas` from the product drawer.
An existing executable there can be switched to the bundled command with
native confirmation. The startup setup screen can also link to an existing
custom bin folder. If the folder is already
on your shell's PATH, check that `command -v atlas` resolves to the path shown
by the app. The command continues to work when the app is closed. Use
`atlas open` or `atlas open '<link>'` to open the app from the terminal.
Replacing the bundle at the same path keeps the link working. If you move the
bundle, remove the old link after checking where it points, then enable the
command again from the new location.

## Test with a disposable workspace

The debug build accepts an alternate home for Atlas subprocesses. Use an empty
directory to exercise setup from scratch, or a disposable copy of a stopped
workspace to test upgrades. Quit the app before switching between test homes.

```sh
test_home="$(mktemp -d)"
open -n 'target/debug/bundle/macos/PF2e Atlas.app' \
  --env "ATLAS_PILOT_HOME=$test_home" --env "PATH=/nonexistent"
```

Use the same `HOME` when you invoke the bundled CLI to test service sharing or
standalone commands. See the
[desktop service boundary](../docs/architecture/decisions/0029-local-web-app-boundary.md#macos-desktop-host)
for ownership and transport details.

From a macOS desktop session, run `node desktop/smoke-macos.mjs` against the
built debug bundle. The smoke test creates a fresh temporary workspace and uses
one local fixture record. It verifies setup opens Search in the same native
window. It also forces a failed render and a failed readiness check to verify
that retry opens the requested record in that window. Each case runs offline and
pauses after the keyword commit, then cancels through the product panel and
verifies that editing is available again. A separate managed-source fixture
checks toolbar and menu access, shared cancellation, a no-op check and a
publication observed in two windows. Embedding generation is skipped.
