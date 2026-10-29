# Validation

Run focused checks and inspect the scoped diff before logical commits. Independent
review is a pre-PR gate, not a commit gate. Finish implementation, resolve
independent subagent review findings, then run the applicable final local gates
before opening the PR. Open an early draft only when the user explicitly requests
one. Respect the tracked hooks. Reuse successful checks for unchanged inputs;
a commit alone does not invalidate evidence for identical inputs.

## Local Rust gate

Rust workspace tests are the primary validation surface. Add or update behavioral
tests under the owning crate, especially for ingest, indexing, lookup, search,
filter discovery, graph context and artifact validation.

For Rust changes, run the full gate with the compiler pinned in
`rust-toolchain.toml`. Rustup-managed Cargo
selects that toolchain automatically; standalone installations such as Homebrew
use their own compiler. Check `rustc --version` and put the pinned toolchain on
`PATH` when necessary. Validation with a different compiler is separate evidence.

```bash
just verify
# or: scripts/verify.sh
```

Validation is quiet by default: successful gates print summary lines and detailed
Cargo output is replayed on failure. Use `just verify --verbose` or
`scripts/verify.sh --verbose` to stream the complete command output.

## Local frontend gate

During frontend iteration and before pushing, run the fast checks and focused
behavioral tests. These catch invalid test APIs and type errors without waiting
for the complete suite:

```bash
just web-ui-check
npm --prefix web/atlas-ui run test -- src/features/encounters/encounterViews.test.tsx
```

Run focused tests for the actual changed owner; the encounter example is not a
universal smoke test. Keep related validated commits in a coherent publication
batch: PR selection uses the complete
base-to-head diff, so another push can repeat gates for earlier unchanged work.
Reuse successful checks while their inputs remain unchanged; after substantive
changes, rerun affected checks and require final-head CI before merge readiness.

At a coherent final candidate, run the full frontend gate:

```bash
just web-ui-verify
# or: npm --prefix web/atlas-ui run verify
```

`verify` performs route preparation and TypeScript checking once, then runs
Vitest and the production bundle. `check`, `test` and `build` remain self-contained
standalone commands. `verify:bundle` performs checks and builds without unit tests;
CI uses it alongside the independent unit-test job. Vite retains its production
router generation/code splitting; the separate Vitest configuration needs no
route watcher or splitter.

`vitest.config.ts` explicitly assigns reviewed value/request tests to the `node`
project. Other tests default to the `dom` project with JSDOM, browser storage and
React Testing Library setup. Review runtime imports and exercised behavior before
moving a file to Node: a URL codec is suitable, but a `.test.ts` suffix alone is
insufficient (storage preferences and `renderHook` tests still need JSDOM).
Run a project directly with
`npm --prefix web/atlas-ui run test -- --project node` or select a file normally.
Both projects share the existing one-worker limit; environment selection does not
relax assertions or remove cases.

## Local browser checks and the CI matrix

Local browser validation targets the changed behavior and affected engine/device
combinations. Broaden coverage for shared layout, navigation, overlays, touch/focus
behavior or browser-harness changes. CI owns the full applicable device/browser
and platform matrix by default; a complete local matrix is available for debugging
or changes that specifically require it, rather than a routine pre-PR gate.

Install the browsers needed for the selected local checks, then run the relevant
spec/project. For example, from the repository root:

```bash
npm --prefix web/atlas-ui exec -- playwright install chromium webkit
npm --prefix web/atlas-ui run test:devices -- e2e/devices/turn-reminders.spec.ts --project=phone-webkit
```

This is an example selection, not a universal smoke test. To run the complete
maintained device acceptance suite deliberately:

```bash
npm --prefix web/atlas-ui exec -- playwright install chromium webkit
npm --prefix web/atlas-ui run test:devices
```

Concurrent worktrees must use separate device fixture ports, for example
`ATLAS_DEVICE_PORT=18867 npm --prefix web/atlas-ui run test:devices`. The suite
checks the served frontend against that checkout's build before every test.


The command builds the frontend and launches `atlas-web`'s developer-only
`browser_fixture` example on 127.0.0.1:8796. The first run compiles Rust using the
normal checkout-local target; later runs reuse it. Port 8796 must be free: the
suite never adopts an unrelated running server. It creates a small checked-source
persistent SQLite workspace under `scratch/responsive-validation/runs`.
Graceful SIGINT removes the default temporary run directory. A forced kill may
leave a run directory for inspection; remove only that inactive run after retaining
needed evidence. No Foundry clone or embedding models are downloaded.

The fixture uses real index, retrieval, app-service and web owners. Structural
stored vectors enable browsing/Similar without claiming inference or ranking
quality; query-asset unavailability remains truthful. The bounded Chromium/WebKit
phone/tablet/desktop matrix exercises real touch and keyboard flows. It is separate
from `npm --prefix web/atlas-ui run test:e2e`, whose broader scenarios require a
caller-supplied real corpus server.

Browser project selection is explicit in `playwright.devices.config.ts`:

| Scenarios | Projects | Protection |
| --- | --- | --- |
| `journeys` Search pagination URLs/history, list query/include/exclude tags, encounter archive history, unreadable-filter recovery (`@engine`) | desktop-chromium, phone-webkit | Both engines; reload/copied links, native history and persisted state |
| `list-metadata` measurement/disclosure (`@engine`) | desktop-chromium, phone-webkit; each checks the 390px phone viewport | MutationObserver catches visible measurement chips; complete long tags, popup bounds, Escape and focus |
| Other journeys, feedback, spell effects, conditions/refinement, encounter forms and turn reminders | All four existing desktop/phone/tablet projects | Original touch, geometry, scrolling, focus, theme, motion and visual checks |
| Compatibility and transition specs | desktop-chromium | Original desktop-only integration checks |

Use `@engine` only for an explicitly reviewed scenario whose meaningful failures
are covered on the two selected engines. Keep viewport-sensitive cases on their
existing projects; do not tag a whole spec merely because some tests exercise
URLs. New untagged cases retain the spec's original matrix. Inspect assignments
with `npm --prefix web/atlas-ui exec -- playwright test --config web/atlas-ui/playwright.devices.config.ts --list`
from the repository root, or run the equivalent command inside `web/atlas-ui`
with `--config playwright.devices.config.ts`.

## CI execution and authenticated browser artifacts

CI builds the frontend once after fast checks. Full Rust verification and a
browser-fixture build then run independently. Unit tests can start alongside the
frontend checks/build because they do not consume the bundle. Two browser shards reuse
that compiled fixture; each runner starts its own disposable SQLite workspace and one
Playwright worker. No mutable database or live Cargo target is shared between
runners. The
`validation complete` check requires both shards, the fixture build, the frontend
checks/build, unit tests and every other selected gate. Main/nightly retain all
surfaces. Frontend `verify` alone does not execute browsers. Browser dependencies
use the Ubuntu HTTPS archive, with a three-minute installation limit; an installer/network failure can rerun just its browser
job with `gh run rerun RUN_ID --job JOB_ID`, reusing the retained compiled fixture.
Use the `databaseId` from `gh run view RUN_ID --json jobs` as `JOB_ID`.

The fixture artifact contains the executable, its expected frontend index and a
manifest with source commit, platform, architecture and SHA-256 checksums. Set
`ATLAS_BROWSER_ARTIFACT` to that directory to run it without Cargo. The runner
rejects another commit/host or corrupted files; every test also checks served
HTML against that artifact's index. CI artifacts expire after three days. Rebuilding the same candidate replaces its
intermediate bundle/fixture; report artifacts include the run attempt so failure
evidence remains available. Local
runs default to the checkout-local Cargo build. Packaging freezes the executable
and expected UI index, but test/configuration files still load from the checkout.
Keep that checkout and its dependencies fixed for the duration of a final run;
changing HEAD can trigger the commit guard, and editing tests can mix versions.
Do not remove that guard or claim a run validates a newer candidate. A separately
managed isolated final-validation runner remains future tooling work. Never point
a candidate test at a frozen preview binary just because it starts successfully.

Rust verification writes a dependency-only cache shared with the fixture build;
the fixture job only restores it. The action keys compiler identity, lockfiles,
Cargo configuration and relevant environment. Workspace crates remain rebuilt,
CI incremental compilation stays disabled, and no Cargo target is transferred as
a test artifact. GitHub's branch scoping and eviction bound retained caches.
Compare job/step timings on cold and warm runs before claiming a speedup.

## Focused Linux browser diagnosis

After a failed device run, repeat the failing spec/project against that run's
compiled candidate instead of rerunning all gates:

```bash
gh workflow run browser-debug.yml --ref feat/my-branch \
  -f browser-run=RUN_ID -f file=turn-reminders.spec.ts \
  -f project=phone-webkit -f grep='recovery' -f repeat=5
```

Use a run from this repository with an unexpired `browser-fixture` artifact. The
manual run checks out the artifact's exact source commit and uses the same Ubuntu
24.04, Node 22 and lockfile-pinned Playwright installation as ordinary CI. It runs
only the selected case, with traces/screenshots/report retained even on failure;
it does not replace the full PR check or emit its `validation complete` result.
The `browser-debug.yml` workflow must be registered on the default branch before
GitHub permits direct dispatch. It owns read-only Actions access for replay;
main/nightly verification keeps its own permissions and checks. For local diagnosis, pass the
same spec, `--project`, `--grep` and `--repeat-each` arguments to Playwright directly.

## Visual references

Device runs always compare the condition refinement checkpoints with tracked
references under `web/atlas-ui/e2e/devices/visual/<platform>/<project>/`. Exact
pixels, viewport, theme and fixture state are preserved; interaction tests also
check focus, geometry, retained anchors and delayed transitions. References are
platform-specific. Missing or changed references fail ordinary runs, and CI
uploads actual/expected/diff images and traces for review. Local evidence lives
under `scratch/responsive-validation/browser`.

To propose Linux references deliberately, dispatch the focused runner with
`-f file=condition-refinement.spec.ts -f project=PROJECT -f repeat=1 -f capture=true`.
Use `-f project=all` to capture every project. This captures candidate files in the evidence
artifact; it never commits them or changes the ordinary gate. Inspect the rendered
states and differences against the accepted reference before copying reviewed
PNGs into Git. For macOS use the same focused command locally with
`--update-snapshots=all`. Do not refresh references to make a refactor pass.
Record source/browser/OS provenance in the visual directory README. A deliberate
CSS perturbation should fail the comparison when verifying this guard.

Follow the frontend guidelines for visual review and actual-device checks;
passing emulation does not certify a physical phone's software keyboard, browser
chrome or text-enlargement behavior.

## Changed-path selection and hooks

For path-sensitive formatting or explicit full validation (the range below assumes a PR against `main`; substitute the actual PR base for integration or stacked work):

```bash
just verify-changed --staged
just verify-changed --range origin/main...HEAD --list
just verify-changed --range origin/main...HEAD --full
```

The selector prints its decisions with `--list` without running checks. Local
`--full` dispatches to the owning validation surface; install that surface's npm
dependencies explicitly first. Rust workspace builds also require the frontend
bundle: the selector prepares it before Rust checks, reusing the frontend gate’s
build when both surfaces are selected. It never installs dependencies as part of
a gate.

| Changed surface | Local full validation / PR CI coverage |
| --- | --- |
| Rust | Workspace gate; CI also checks Windows storage and standalone atlas-dev |
| Frontend | Frontend gate; CI also runs Rust verification and real-app browsers |
| App-model contracts | Both Rust and frontend |
| Foundry model | Rust and source-contract generation |
| Source-contract tooling | Fixture tests and pinned generation |
| Release tooling | Tool tests, notices and installer checks; CI includes three OSes |
| Research datasets, including hash-bound Markdown | Each dataset's offline `verify.ts`, with no inference |
| Prose documentation | Diff/links/examples review; no application builds |
| Cargo manifests/lockfile | Rust, frontend, source contracts and release/notices |
| Shipped agent skill | Metadata and bundled-file coverage checks (`node scripts/validation/skill.ts`); no application build |
| Unknown paths, workflow or validation infrastructure | All surfaces |

PR CI uses the same path classifier. Renames include the old and new paths;
executable/symlink mode changes select all. Main pushes and nightly verification
run every surface. The `validation complete` job rejects failed or unexpectedly
skipped selected jobs; use it as the aggregate required check. CI selection and
local script stages report decisions/timings in the job summary. Two Clippy passes
retain distinct lint policies; tests and normal builds remain separate until
measurement establishes equivalent coverage at lower cost.

`test:devices` builds the frontend for standalone local use. CI invokes Playwright
directly against the authenticated fixture artifact, without rebuilding Rust or
the frontend in each shard.
`just verify-all` validates installed Rust/frontend dependencies without rerunning
`npm ci`; use `just web-ui-install` when the lockfile or installation changes.

Tracked git hooks live in `.githooks/`:

- `pre-commit` and `pre-merge-commit`: check staged whitespace and run path-sensitive formatting checks (Rust and/or frontend). Docs-only changes need only the whitespace check.
- `commit-msg`: require a Conventional Commit subject; an optional body must be blank-line-separated.
- `pre-push`: check third-party notices when outgoing commits change Cargo manifests/lockfiles, notices, Cargo configuration, or release tooling. New refs are checked conservatively. This rebuilds the small TypeScript release tool and runs Cargo metadata offline; it does not compile Rust, install packages, or rewrite notices. Install dependencies explicitly with `npm --prefix dev-tools/release ci --ignore-scripts`. For stale notices, run `npm --prefix dev-tools/release run notices` and commit the result. Push relevant refs from a checkout at their tip with committed dependency/notices inputs so the check validates the published candidate.
- Run the applicable local gates explicitly before opening a PR, reuse passing evidence for unchanged code, and require green CI on the published head before merging. CI retains Clippy, typechecking, tests, builds and platform/browser checks.

Formatting checks read files in the working tree; whitespace checks read the staged diff. Review partial staging carefully so local evidence corresponds to the committed candidate. Hooks do not replace code review or CI.

## Database changes

Domain-owned SQL is the physical schema source of truth: reference SQL lives in `crates/atlas-index/migrations/`, saved-domain SQL in `crates/atlas-local-state/src/schema.sql`. Runtime composes the ordered workspace migration ledger; storage executes it with checksums, backup and atomic validation. Once a migration is released, append an upgrade rather than editing its applied SQL. Keep tests for failed-upgrade rollback and saved-state preservation. Query-only changes can use the supported DTO-based projection backfill; do not force source admission or inference for schema changes alone.

Docs-only or instruction-only changes do not require the Rust or frontend gates unless they affect executable examples, generated output expectations, or another validated artifact. Inspect the diff and check whitespace and any changed links or examples.

## Merge readiness

The normal landing path is a reviewed GitHub PR, with applicable local gates, CI, and human acceptance reported separately. `scripts/land-worktree.sh` and `just land-worktree` directly fast-forward local `main`; they are not the normal PR workflow and require explicit authorization to use. Do not interpret a passing gate as permission to merge.
