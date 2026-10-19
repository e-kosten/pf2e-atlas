# Dice evaluator prototype research

2026-10-17. This is an early UX prototype, not a production dependency decision.
No encounter state, artifact data, or Foundry source is modified.

## Presentation alternatives

The prototype chooses lightweight SVG dice silhouettes with CSS animation in
fixed inline slots. Faces come only from the Rust receipt. There is no physics
simulation, animation framework, or independent browser dice evaluator. This
small product-specific component is an interaction experiment, not an adopted
design system. Immediate mode and system reduced motion show the same results.

[RPG Dice Roller](https://github.com/dice-roller/rpg-dice-roller) is a MIT JS
alternative with detailed result APIs and configurable randomness, but was not
installed in this spike: it would require deciding how Rust CLI/backend callers
share execution. [Dice Box](https://github.com/3d-dice/dice-box) adds BabylonJS,
physics, workers and static assets; its README describes simple dice input and
results after physics settles. [Dice Box ThreeJS](https://github.com/3d-dice/dice-box-threejs)
explicitly documents supplied outcomes (`6d6@4,4,4,4,4,4`), so backend authority
does not prevent a future3D presentation. Neither renderer was installed or
benchmarked. The approved preference for subtle inline animation does not need
their dependencies for this first checkpoint; actual3D appearance/performance
remains untested. Keep animation replaceable without altering result semantics.

## Evaluator comparison

| Candidate | Evidence | Prototype decision |
|---|---|---|
| Caith 4.2.4 (MIT) | Public roll history contains individual values; arithmetic and keep-high/low supported; injectable seeded RNG. Direct dependencies are pest, pest_derive, rand0.8. | Use for bounded presets. |
| DiceXp 1.3.0 (MPL-2.0) | Public DiceRoll contains total/min/max/average, without individual faces. Injected RNG supported. Direct dependencies include rand0.10 and bytemuck; optional CLI/serde features. | Actual test probe passes mixed expression but rejects Caith-style keep notation. Result API unsuitable for our animation breakdown without extra machinery. |

Primary sources: [Caith documentation](https://docs.rs/caith/4.2.4/caith/),
[Caith source](https://docs.rs/crate/caith/4.2.4/source/src/),
[DiceXp source](https://docs.rs/crate/dicexp/1.3.0/source/src/lib.rs),
[DiceXp manifest](https://docs.rs/crate/dicexp/1.3.0/source/Cargo.toml).
The corresponding downloaded registry sources were inspected, compiled and probed.
Licenses above are package metadata, not legal advice about redistribution.

## Important Caith limitations found in source and probes

- History sorts values within each dice pool; it does not preserve physical roll order.
- Each die result contains its value and minimum/maximum marker, not its side count.
- Keep-high/low retains original faces but does not directly mark discarded dice.
- A preset supplies the known sides. For the two-dice keep-one presets, the
  library-computed total minus the known modifier identifies the kept value;
  exactly one matching die is marked, including ties. Atlas does not independently
  calculate the roll total or parse arbitrary notation.
- This is sufficient to evaluate the proposed UX. It is not sufficient evidence
  for accepting arbitrary expressions in production. A richer AST/result model,
  another library, or upstream additions should be compared before that decision.
- The library's advertised dice/explosion bounds are much broader than this
  prototype needs. This API admits a small explicit preset vocabulary, optional
  signed integer modifiers from -100 through100, at most32 groups,100-byte labels,
  128-byte IDs,40-byte formulas and a16KiB request body. It rejects all other syntax.

Foundry expressions are not generally portable: references, damage annotations,
PF2e substitutions and rules context must be resolved deliberately by an eventual
application owner. No current prototype result claims full Foundry compatibility.

## API and animation boundary

Run from the task worktree:

```sh
cargo run --manifest-path dev-tools/dice-prototype/server/Cargo.toml --target-dir target --locked
```

Listens only on127.0.0.1:8821. The frontend development server proxies `/roll`
and `/health`; this server does not enable unrestricted CORS.

```sh
curl http://127.0.0.1:8821/roll -H 'Content-Type: application/json' \
  -d '{"id":"example-1","groups":[{"label":"Slashing","formula":"1d8+4"},{"label":"Fire","formula":"1d6"}]}'
```

Response: id, groups(label,formula,total,dice[sides,value,kept],modifier,breakdown),
and summed total. Batch initiative uses the independent group totals, never the
summed total. Groups also keep damage types separate. The browser animates the
returned faces; it does not calculate random outcomes.

Accepted bases:1d4,1d6,1d8,1d10,1d12,1d20,1d100,2d6,8d6,1d8+1d6,
2d20K1(keep highest),2d20k1(keep lowest), each optionally with a signed modifier.
Whitespace is ignored. These are prototype presets, not a promised product DSL.

`delay_ms`0–5000 and `fail` are explicit development-only fault controls. Failure
happens before rolling. Same-ID/same-groups retries return identical cached results;
changed groups with a retained ID return409. Cache holds512 recent successful
requests in process memory; eviction or restart loses this guarantee. Production
idempotency needs durable workflow design. There is no save or turn advancement.

## Validation

Six focused tests pass:

- Mixed d8/d6 retain sides and their sum plus modifier matches library total.
-500 seeded trials each for keep-high and keep-low retain both faces and exactly
 one kept face, even ties, with a nonzero modifier.
- Unbounded/nonpreset/Foundry expressions rejected.
- Retrying retains outcomes and a changed payload conflicts.
- DiceXp compiled probe confirms its numeric result shape and notation limit.
- Plain-text breakdown keeps discarded faces visible without library Markdown.

`cargo fmt` and strict all-target Clippy pass. Local API smoke returns the expected
structured response. Task target was327MiB after these checks, with no workspace
build or corpus/model rebuild. Broader maintenance/security review and arbitrary
expression support are outside this early prototype decision.

## Animation comparison — installed and exercised

The standalone preview now uses @3d-dice/dice-box 1.1.4 and
@3d-dice/dice-box-threejs 0.0.12 (both MIT), pinned in renderer-deps/package-lock.json.
It retains the custom SVG/CSS option for comparison. Official documentation:
https://fantasticdice.games/docs/usage/config and
https://github.com/3d-dice/dice-box-threejs .

ThreeJS accepts supplied faces using its @ notation. Actual browser checks cover
single d20, 2d6, d8+d6, keep-high/low (animate both, retain selected die in result),
and 8d6. Every returned face must match the backend receipt. Dice Box's roll()
resolves a flat array of die results; it generates independent faces from physics.
Its getRollResults() has a different grouped shape. The preview validates count,
sides and bounds, then applies only the fixture's modifier/keep choice.

Early Chromium click-to-settled timings: custom approximately 0.7–0.85 s,
ThreeJS 2.2–3.2 s, Dice Box 2.4–4.1 s. These include backend transit and presentation,
are not controlled benchmarks, and do not establish mobile/Safari performance.
Installed dependencies total 94 MB; static production preview output 18 MB.
No full Rust workspace, corpus or embedding build was needed.

Architecture remains open. ThreeJS shows backend outcomes are compatible with
real physics animation. Choosing Dice Box would favor browser-generated faces,
possibly submitted to Rust for validation and rule application. A production
contract would need requested-dice identity, per-face bounds/count validation,
retry/idempotency and an explicit authority for modifiers and keep rules. It
should not accept an unchecked browser total or calculate conflicting outcomes
in two places. CLI/batch rolls could still use a backend evaluator. The preview
adds no frontend-result persistence API and no encounter mutations.

Still to assess after UX feedback: library lifecycle in a production component,
rendering load on real devices, roll fairness/PRNG expectations, accessibility,
error recovery, asset packaging and maintenance implications. No final selection.

## Compact in-place alternatives (2026-10-17)

User rejected travelling physics-tray presentation, not necessarily 3D geometry:
short in-place rotating cubes/polyhedra are acceptable. Backend-standardized
outcomes remain preferred. No new libraries installed during this research pass.

- Obsidian-TTRPG-Community/dice-roller: src/rollers/roller.ts setSpinner() inserts
  Icons.LOADING inside .should-spin; src/assets/main.css animates it with CSS
  rotation keyframes. This is plugin-local code, not a reusable animation package.
  It may be the remembered compact interaction; identity not confirmed by user.
  https://github.com/Obsidian-TTRPG-Community/dice-roller/blob/main/src/assets/main.css
- react-dice-complete 2.3.2 (ISC): reusable CSS cube animation, configurable
  rollTime and rollAll(values) for predetermined results. README explicitly says
  visual display always d6 even with other side counts. Current package declares
  React peers through18, not Atlas19. Useful visual comparator, not a complete
  polyhedral renderer or verified drop-in dependency.
  https://github.com/AdamTyler/react-dice-complete
- ChefJulio/react-3d-dice (MIT): fixed-grid spinning d4/d6/d8/d10/d12/d20, supplied
  results prop, full/quick/none modes, configurable height, no tray physics.
  Best new visual candidate under clarified requirement. Source Dice3D.jsx has
  perpetual requestAnimationFrame and idle wobble/bob even in none mode; quick
  changes spin speed, caller controls rolling duration. One sides value per
  component; mixed dice need composition. Small project (four commits in inspected
  repository snapshot); not browser-tested here. Production needs motion/lifecycle
  assessment; do not imply it is turnkey or actually respects reduced motion.
  https://github.com/ChefJulio/react-3d-dice/blob/main/src/Dice3D.jsx
- DiceForge renderer-web (MIT, experimental): accepts resolved records, offers
  DOM mode. Direct source inspection shows labelled64px tiles scaling in over
  320ms and subsequently dimming dropped dice; not spinning die shapes. Its DOM
  branch sets120px minimum height and16px padding and the package includes Three.js.
  Lower priority for this visual goal than its marketing description first suggests.
  https://github.com/StephanTTM/DiceForgeSDK/blob/main/packages/renderer-web/src/dom/backend.ts

Recommendation: keep backend evaluation; next preview compares fixed-position
polyhedra (react-3d-dice) and CSS cubes (react-dice-complete) against existing inline
SVG. Evaluate exact face agreement, mixed dice, short duration, idle/reduced-motion
behavior, sizing and dependency compatibility before selecting a production path.
No adoption decision or new production implementation implied by this shortlist.

## Follow-up decision and experiment

User requires well-maintained OSS for production adoption, but authorized retaining
ChefJulio's package solely as a visual experiment/inspiration. Installed exact
react-3d-dice0.1.0 in isolated renderer-deps. It is a distinct project from
https://github.com/aqandrew/react-3d-dice (API pushed_at2024-02-08; no repository
license detected). ChefJulio API pushed_at2026-02-12, four commits in inspected
history. Recency alone does not establish ongoing maintenance. Installed package
metadata points to ChefJulio/react-dice-3d; source project is react-3d-dice.
No production adoption recommendation.

New default comparison: original tumble, OSS fixed-position polyhedra, and custom
spinning SVG icons inspired by Obsidian's interaction (not copied plugin code).
All use one backend receipt, preserving kept/dropped dice and modifiers. Added
separate silhouettes for d4,d10,d12, plus existing d6,d8,d20. OSS mixed dice compose
one library component/canvas per die; remaining production concerns include this
cost and idle animation. Its animationMode none does not stop idle motion, so
prototype reduced-motion mode renders static SVG instead. Existing tray experiments
remain accessible separately. User preview precedes broader validation/review.

## Production evaluator decision

Production uses MIT-licensed [diceng](https://github.com/Syrup/diceng), pinned to
2.0.0 with default features disabled. The packaged source identifies commit
109a4d57070ae38b65958e3e599ca6c282fb8941. Its changelog records parser fixes in
1.0.2 (2026-06-17) and structured result improvements in 2.0.0 (2026-06-20).
The upstream repository exposes tests and CI. This is evidence of recent
maintenance, not an established long-term support guarantee: the reviewed GitHub
snapshot had a small history (19 commits). Keep the dependency isolated in
atlas-dice and reassess maintenance when upgrading.

The production choice replaces the prototype's preset-dependent Caith projection:
diceng supplies a parse tree, bounded evaluation inputs, seeded RNG for tests,
and structured original dice pools. Atlas validates the supported arithmetic and
keep-high/low subset before calling the evaluator, then projects actual faces and
kept state without reparsing formulas or determining results in JavaScript.
Tests cover original draw order, ties, max faces, arithmetic, unsafe bounds and
mixed pools. All 589 nonempty strike formulas in the pinned corpus passed prepare;
the sole empty formula describes a nondamaging grab/pull attack.

The production visual comparison is at `/production.html` on the prototype
server. It imports the actual shared DieFace implementation and uses fixed outcomes
for all six drawing/motion combinations, including maximum values and nonstandard
dice. Earlier experiments remain separate from production dependencies.
