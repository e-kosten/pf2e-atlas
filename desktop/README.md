# Develop the desktop app

## Windows x64

Use the pinned [Rust toolchain](../rust-toolchain.toml), Node 26, and the
Microsoft C++ v143 Build Tools with the Windows SDK and x64 redistributable.
Build from a Windows x64
PowerShell session after installing those [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/#windows).

```powershell
npm --prefix web/atlas-ui ci
npm --prefix desktop ci
npm --prefix dev-tools/release ci --ignore-scripts
./desktop/bundle-windows.ps1
```

The unsigned per-user setup executable is in
`target/x86_64-pc-windows-msvc/release/bundle/nsis`. It includes the matching
`atlas.exe` and the app-local Visual C++ runtime. The installer supplies
WebView2 if needed. The Windows desktop CI workflow
installs the package on a clean runner, checks the bundled CLI and protocol
registration, and uploads the installer as a build artifact.

Start **PF2e Atlas** from the Start menu. Choose **Set up Atlas** to prepare a
new workspace. In the Windows tabs prototype, use **File → New Tab** (Ctrl+T)
to bring up the tab strip. Ctrl-click, middle-click, and **Open Link in New Tab**
open Atlas links in background tabs. Drag a tab to reorder it. Hold it near either
edge of a crowded tab strip to scroll toward the first or last tab. To reorder a
focused tab with the keyboard, press Alt+Shift+Left or Alt+Shift+Right. Ctrl+Tab
and Ctrl+Shift+Tab switch tabs.
Ctrl+W closes the selected tab, and
Ctrl+N opens a separate window. Tabs and windows share the local service and
workspace. Atlas links opened from another app select an open matching tab or
open the destination in a foreground tab. **Terminal and Agent
Access** in the app menu shows the path to the bundled command for PowerShell.

Test installation and first run from a disposable Windows user account or VM.
Check that setup opens Search in the same window, a second window shares saved
state, and a copied Atlas link opens the correct destination both while the app
is running and after it exits. Close the last product window and reopen the app
to verify it reuses the prepared workspace.

With a creature selected in a running encounter, press **Ctrl+F** and enter
an offscreen statistic such as **Speed**. Verify that WebView2 highlights
the text and brings its value into view. Move through a repeated word in both
directions, then press **Esc** and check the selected creature and turn.
In a second tab, search for a phrase specific to the first tab’s creature
description and check that the hidden page contributes no matches.

## macOS build

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

For the adaptive Dock and Finder icon, build with full Xcode 26 or later selected.
You can select it for a single build with
`DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer sh desktop/bundle-macos.sh`.
A local build with Command Line Tools uses the static icon. The release DMG
requires the adaptive icon catalog.

Edit the layered artwork in [Atlas.icon](src-tauri/icons/Atlas.icon) with Icon
Composer. Keep its compass geometry aligned with the web favicons. If the
Default artwork changes, export it again and regenerate `Atlas.icns` for the
static fallback.

## Use the app

Choose **Set up Atlas** when the workspace is empty. The app downloads the
pinned reference data and opens the workspace in the same window as soon as
the keyword index is ready. You can browse and search while Atlas prepares
semantic search. Saves are paused across Atlas sessions during setup and resume
when setup ends. Select the setup activity in the toolbar to see progress or cancel.
If you cancel after keyword search is ready, use **Resume semantic setup** to
continue later. If precomputed embeddings are unavailable, choose
**Retry download** or **Build embeddings locally**. The local build can take an
hour or more. When semantic search becomes ready, the next submitted search uses
it by default. Use **Rerun this search** to apply it to the current keyword results.
If the view fails to open, choose **Retry opening Atlas** to
use the prepared workspace. If Atlas asks you to upgrade
a workspace, close any other Atlas sessions using that workspace and choose
**Update workspace**.

On macOS, **PF2e Atlas → Settings…** (⌘,) opens a separate window. Use its
**Search & data** pane for workspace maintenance. On Windows, open **Settings**
(Ctrl+,) and choose **Search and reference data**. Close the Settings tab to
return to your work.
Each Atlas build pins a Foundry PF2e revision. After installing a build with
a different pin, confirm **Apply bundled reference data** to prepare that
build's reference data. Use **Check and repair reference data** to check the
current pin and prepare any missing assets. Atlas keeps your selected record
and drafts open. If the reference data changed while Search was open, use
**Refresh results** when prompted to read from the new version. For a custom
source, use the Atlas CLI to maintain its reference data.

On macOS, use **File → New Tab** (⌘T) or **File → New Window** (⌘N) to open a fresh
Search view. Command-click, middle-click, or choose **Open Link in New Tab**
from an Atlas link’s context menu to open its destination in a background tab.
All windows and tabs share the local workspace and service with a compatible
`atlas web` session. Choose **Quit PF2e Atlas** to release the app’s connection
to the service.

Press **⌘F** on macOS or **Ctrl+F** on Windows to find text on the current
page. In a running encounter, try **Speed** or **Fortitude** to bring that
statistic into view. Use the arrows to move between matches and **Esc** to
close Find. On macOS you can also choose **Edit → Find…**.

Use **View → Zoom In**, **Zoom Out**, or **Actual Size** to scale the entire
view. Atlas saves one zoom level for all windows and tabs.

Use the link icon in an entity header, or choose **Edit → Copy Atlas Link**
for the page's main entity. On a list page, the Edit menu copies the list link.
Use the reader's icon to copy the record link. The menu item becomes available
when the entity loads.

Paste into a rich-text editor for a linked entity name or into a plain-text
field for the `pf2e-atlas://v1/…` URL. Atlas links continue to work if the local
service moves to a different port. Saved list and encounter links use stable
keys and require that entity in the active workspace. In the browser, the same
icon copies the entity's web URL.

Opening a record link from another app goes to the standalone record page.
An existing tab on that page is selected. Search and list previews stay in their
own tabs. On macOS, you can also use `atlas open '<link>'`. Atlas opens a new tab
in the most recently active window when the page is not open, or opens a window
directly to the destination.

On macOS, open **Settings → Terminal** to link the bundled command at
`~/.local/bin/atlas`, or select **Choose bin folder…** to use another
existing directory. During setup, use **PF2e Atlas → Terminal and Agent Access**.
Atlas asks for confirmation before replacing an existing executable. If the folder is already
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

Run the bootstrap and bundle configuration tests with
`node --test desktop/bootstrap.test.mjs desktop/release-config.test.mjs`.

From a macOS desktop session, run `node desktop/smoke-macos.mjs` against the
built debug bundle. The smoke test creates a fresh temporary workspace and uses
one local fixture record. It verifies setup opens Search in the same native
window. It also forces a failed render and a failed readiness check to verify
that retry opens the requested record in that window. Each case runs offline and
pauses after the keyword commit, then cancels from Search & data in Settings and
verifies that editing is available again. A separate managed-source fixture
opens Settings from two workspace windows, checks that both use the same
Settings window, and exercises cancellation, a no-op check and reference
publication. The link scenario sends real macOS URL
events to an isolated app bundle. It checks a cold launch, a warm open with the
same record in a Search preview, and reuse of the standalone record tab.
Embedding generation is skipped.

## Qualify Windows updater transitions

After pushing a candidate branch, run the disposable native updater rehearsal
with the GitHub CLI. Replace `BRANCH` with that branch:

```sh
gh workflow run windows-desktop.yml --ref BRANCH -f qualify-updater=true
```

This optional workflow builds test installers with disposable updater signatures
and serves them over local HTTPS on a fresh Windows runner. It drives the installed
app through WebView2, verifies that NSIS replaces both the host and CLI after
the old processes exit, and checks separate-copy startup and refusal after
database identity or migration-history changes during installation.

The test installer uses a guarded restart worker to preserve its isolated data
and browser-profile paths. It exercises target startup admission; it does not
qualify NSIS's ordinary `RunAsUser` restart environment. The default installer
workflow and published packages use their normal restart path.

Inspect the run's `windows-updater-qualification-*` artifact for screenshots,
process/digest checks and refusal evidence. A passing run includes `result.json`
with all three cases completed. The artifact excludes signing keys and test
databases. The scripts require a disposable GitHub Actions runner and refuse a
runner with existing Atlas data or a running Atlas app.
