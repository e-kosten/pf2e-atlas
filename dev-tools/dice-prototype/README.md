# Dice interaction prototype

## Production component comparison

Open `/production.html` on this development server to compare the actual shared
product dice component across simple/shaded drawings and spin/tumble/rock motion.
It includes maximum faces, nonstandard sides and discarded dice, with fixed
outcomes and no gameplay mutations. Production defaults are shaded and spin.

The remaining pages and notes below preserve the exploratory comparisons that
informed the implementation. Their evaluator server and physics dependencies are
development experiments; the application uses `atlas-dice` and the shared UI
components. Production validation includes workspace gates, responsive browser
journeys and independent visual review, separately from these prototype checks.

## Exploratory prototype

Early human-review checkpoint, not production integration. Open
http://127.0.0.1:8820. The Rust evaluator listens on8821. No reference database,
saved encounter or model download is involved. Reload clears UI state and history.

Try individual and batch initiative; choose Stealth for the Ogre Warrior to see
an unlisted skill, or override any modifier. Identical creatures roll separately.
Attacks & dice covers a single check, separately labelled damage components,
mixed die sizes, keep-high/low and eight dice. Roll table uses actual Foundry
terrain ranges. Animated/Immediate compares presentation; settings adds a slow
response or simulated failure. Retry uses the same request ID. Device reduced
motion suppresses animation. The original workflow tabs calculate in Rust.

Animation comparison shows three real implementations: custom SVG/CSS,
Dice Box ThreeJS 0.0.12 (both receive the same Rust result), and Dice Box 1.1.4
(generates its own faces in browser physics). The comparison includes single,
multiple and mixed dice, keep-high/low, and eight-die pools. Narrow fixture code
adds modifiers and marks kept dice for the browser result; this is not a proposed
production expression parser. The ThreeJS result is checked against every supplied
face before acceptance. Reduced motion/Immediate skips both physics renderers.
The settings' simulated backend delay/failure apply to the original workflow
tabs, not the comparison. Timings measure click through settled presentation.

This is a bounded experiment: not a production rules engine, full encounter
initiative editor, persistent roll ledger, general expression grammar or complete
table draw system. In-memory retry retention is bounded and lost on server
restart. Unlisted skill, modifier and table fixtures do not prove full contextual
PF2e behavior. No target resolution, attack penalties, automatic damage application,
table recursion or draw-without-replacement is implemented. Prototype source is
uncommitted pending early UX feedback; no PR is open for this task.

## Run

From this task worktree, use its one local target:

```sh
cargo run --manifest-path dev-tools/dice-prototype/server/Cargo.toml --target-dir target --locked
```

In a second terminal, from `dev-tools/dice-prototype`:

```sh
npm run dev
```

For the animation comparison, install the separately locked renderer dependencies
and expose their local assets first:

```sh
npm ci --prefix renderer-deps --ignore-scripts --registry=https://registry.npmjs.org
npm run prepare:renderers
```

Each physical renderer lives in an isolated same-origin iframe so leaving the tab
releases its workers, listeners and WebGL context. Libraries and assets stay local;
there are no runtime CDN requests. This is experiment isolation, not a proposed
production component boundary. Public assets are ignored links to npm packages.

Current preview reuses the installed frontend dependencies through a read-only
node_modules link to the filter-counts worktree. Keep that dependency owner while
this preview is running. A fresh checkout needs its package dependencies installed.
The prototype uses the shared Atlas theme directly, not a copied theme.

## Evidence and provenance

- `npm run build`: TypeScript and Vite pass.
- `node smoke.cjs`: real Chromium interactions, desktop/390px layout, structured
  dice/group attribution, keep/drop, table match, failure/retry, reduced motion.
- `node slow-smoke.cjs`: unlisted Stealth modifier and slow-response animation.
- `node comparison-smoke.cjs`: all six presets across three renderers, including
  supplied-result equality, native result bounds/counts, phone width and reduced
  motion. No browser page errors in the Chromium run. Safari and independent
  Astra visual review remain pending the early UX checkpoint.
- Rust focused tests cover evaluator bounds, mixed dice, keep ties and retries.
- Ant CLI usage lint: no findings.
- Screenshot/evidence files are local under `evidence/`.
- Full gates and independent Astra visual review follow user UX feedback.

Foundry fixture source: local `vendor/pf2e` commit
`f38e97960d53259e7b8461ef7162fe2448beab31`; NPCs in
`packs/pf2e/pathfinder-monster-core/{kobold-warrior,kobold-scout,ogre-warrior}.json`;
table `packs/pf2e/rollable-tables/random-terrain-type.json`. Unlisted NPC skill
baseline follows `src/module/actor/npc/document.ts` prepareSkills, which uses
`skill?.base ?? this.system.abilities[attribute].mod` before modifier adjustments.
Damage example2d6+4 slashing plus1d6 fire is synthetic; it is not a claim about
the Kobold's crossbow. The source-backed Crossbow check is independently labelled.

See [RESEARCH.md](RESEARCH.md) for actual evaluator probes and remaining limits.

## Inline comparison checkpoint

The comparison uses two rows and three columns: Spin, Tumble, Rock. The top row
uses simple silhouettes; the bottom uses shaded static SVG projections. All
receive the same backend receipt. Tilt and the fixed-polyhedral renderer are
removed from the comparison; earlier physics experiments remain separate.

Select “All dice types · d4 to d20” for one of each, or use the fortune/misfortune,
mixed damage and large-pool presets. Immediate/reduced motion skips animation.

The polyhedral assets in src/projected-dice.ts are generated offline with
`node generate-dice-svg.mjs`. Three.js supplies the Platonic solids; d10 uses
a pentagonal trapezohedron. Supporting planes determine polygon faces; hidden
faces are omitted, and directional shading uses face normals. The UI imports
only static projected coordinates, with no live 3D renderer.

`node inline-smoke.cjs` checks row/column order, all six types, mixed/pool rolls,
fortune/misfortune, phone width and reduced motion. `node settle-smoke.cjs`
checks stable positioning across a delayed result. Human feedback on geometry
and motion remains the next checkpoint; independent Astra review is pending.
