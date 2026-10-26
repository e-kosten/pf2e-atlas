# Device visual references

Normal device runs compare exact pixels at ten condition-refinement checkpoints
per project, alongside interaction and transition assertions. References are
separated by OS and project; keep the pinned Playwright version, viewport, theme
and fixture state identical. See CONTRIBUTING.md for deliberate capture/review.

## Bounded HP editor references

The accepted HP layout reserves space for the numeric facts and action buttons.
Narrow panels keep the action group below the values; long formulas scroll
inside the capped editor. Condition, damage, recovery and modal content remain
intact. Exact pixel comparisons retain `maxDiffPixels: 0`.

macOS arm64, Playwright 1.62.1: source
`15e40d7c1006b258c7da79030733e7abeaa058fb` passes all 12 capture scenarios.
Thirty phone/tablet frames change for the HP action row and consequent content,
anchor and overlay-background positions. Parent review covers ten tablet pairs;
independent Astra review covers twenty phone pairs. The ten desktop references
remain byte-identical to the preceding accepted set.

Linux x64, Ubuntu 24.04, Playwright 1.62.1:
[capture 40263640425](https://github.com/e-kosten/pf2e-atlas/actions/runs/40263640425)
passes all 12 scenarios using CI 40262746889's fixture from synthetic merge
`6f4d7cfc1e798f7b23257f9d3769fa4516e3c880`. Its Git tree is identical to the
macOS source above (`bedd7d6f95d662e55b1a3127dfd976197c491688`). Only ten phone
Chromium frames change, independently reviewed by Astra; the other thirty
references remain byte-identical. Binary SHA-256:
`33265b43b692eb49e67f63837fb9c543c0dd075c15678bd3f3e115f2297842c2`;
frontend index SHA-256:
`cc615bc2b95756fc3fc0d0bc6b58e8e466f92bb52aff0959c74ac2abf366c6ba`.
These are separate platform captures, not copied across operating systems.

## Product theme references

### Accepted encounter workflow references

macOS arm64, Playwright 1.62.1: source `5ddfb9be795d8d1784b48675d7ac2c3582685083`
captures the accepted compact roster header, state/turn placement, persistent
Add/initiative footer and aligned pane insets. All 12 capture scenarios pass their
interaction/geometry assertions. All 40 changed before/candidate frames were
reviewed: parent reviewed desktop/tablet pairs, independent Astra reviewed both
phone projects and authenticated their hashes. Foreground condition/edit/recovery
content and controls remain intact. Pair evidence and hashes are retained in
`scratch/pr109-review/reference-pairs/`; the source-default match fix also has a
separate blocked-write browser check. An ordinary comparison run remains required.

Linux x64, Ubuntu 24.04, Playwright 1.62.1:
[capture 40252941816](https://github.com/e-kosten/pf2e-atlas/actions/runs/40252941816)
passed all 12 scenarios using CI run 40252568623's authenticated fixture, PR head
`5ddfb9be795d8d1784b48675d7ac2c3582685083` and synthetic merge
`23e455f1de7f16a9ead21067ff6a3823d1bebee8`. Binary SHA-256:
`52dbf01081eb7977cec02993d9c877bc4bd8e51332a177c9ec573733bb578c34`;
frontend index SHA-256:
`7d87de6ef83fcf9ffadb62d01422f89c165fe819cb83b315cdac1ec812708964`.
All 40 Linux pairs were separately reviewed by the parent (desktop/tablet) and
independent Astra (phones), with hashes authenticated before adoption. Ordinary
hosted comparison remains required; macOS PNGs were not copied to Linux.

The current references capture the accepted Ink dark neutral ramp, bright dark
text family and stronger overlay/control contrast while retaining the
parchment-and-oxblood light theme. All four projects passed their 12 interaction
and geometry scenarios during both captures. Review of every changed frame found
no layout, content, focus-target or overlay-anchor changes: 27 macOS and 33 Linux
frames changed, while the remaining 20 frames are byte-identical to the preceding
product-theme references. Parent review covered all before/candidate contact
sheets; independent presentation review covered all four projects on both
platforms.

macOS arm64, Playwright 1.62.1: captured locally from PR head
`e1874061a458a9471a4282367f307697b55c6625` using all four maintained projects.

Linux x64, Ubuntu 24.04, Playwright 1.62.1:
[capture 40248877625](https://github.com/e-kosten/pf2e-atlas/actions/runs/40248877625)
passed all 12 scenarios using the authenticated fixture from CI 40239066738,
PR head `e1874061a458a9471a4282367f307697b55c6625` and synthetic merge commit
`00082a026c4f02230be5b952969201794581a0dc`. Binary SHA-256:
`e31d99d8ee1f065ad8918d4f42a64b73c5e86b8c0d677c3b9c5ecdbb4e2511f4`;
frontend index SHA-256:
`bf677e939007b4ceda2492f419bebc6809ed6a8fbe899affe30e3f9c4d4e52e2`.
An ordinary hosted comparison with snapshot updates disabled remains the final
acceptance check for the published candidate.

### Pre-Ink product-theme baseline

The preceding references captured the accepted parchment-and-oxblood light theme,
oxblood-and-brass dark theme, workbench layout and revised record identity. The
live product direction was accepted before capture. All 80 macOS/Linux frames
were then reviewed in per-project contact sheets, with the editor, preview,
Quickened, damage and recovery states also inspected at full size. Exact pixel
comparison remains enabled.

macOS arm64, Playwright 1.62.1: captured from source
`3d643cae21f511f9f4c9aef1e055d9a9c5636d09` using all four maintained projects.
All 12 interaction and geometry scenarios passed while generating the 40 frames.
The same candidate also passed the four-project spell-rank keyboard journey.

Linux x64, Ubuntu 24.04, Playwright 1.62.1: [capture 40170724383](https://github.com/e-kosten/pf2e-atlas/actions/runs/40170724383)
passed all 12 scenarios using the authenticated fixture from CI 40169761259,
PR head `7f212c71e527b95feca31b54dfce7cbf161ae6d0` and synthetic merge commit
`4b4edde5f9bf37934c713dda40e0f3fd4a209621`. Binary SHA-256:
`17ee086c535451354e52a3929d6e319f8d14b309f3f3c4b9f403dd1c85449b6d`;
frontend index SHA-256:
`f85d47e55d28e663f69b14cdd93efdaaac59e4ead384bc71f26b1be779bffdef`.
The later keyboard-focus and production chunk-initialization repairs do not alter
these frames. A normal hosted comparison with snapshot updates disabled remains
the final acceptance check for the published candidate.

## Final creature/encounter validation references

The current references use the human-accepted creature/encounter direction plus
reviewed recovery focus and plain Actions-label refinements. An unmodified
three-action allowance has no explanation control; modified/uncertain allowances
keep their calculation without duplicate spending fields or current-turn captions.
Passive hints are deliberately dismissed before the source/fallback checkpoints.
Exact pixel comparison remains enabled.

macOS arm64, Playwright 1.62.1: captured on source
`ae5d763f1ca4ba9396e54612e1c7a26b478b3faa` using the four maintained projects.
All 12 capture scenarios and the subsequent ordinary 12-scenario/40-frame comparison
pass. Thirteen frames change from the preceding `0af3421d` set: parent directly
reviewed nine desktop/tablet pairs, independent Astra reviewed four phone pairs;
27 frames remain byte-identical to their previous reviewed references. Review
hashes and pair evidence are under `scratch/review/creature-final-visuals/r24`
and `scratch/review/creature-final-astra/r24-final-mac-phone-hashes.json`.

Linux x64, Ubuntu 24.04, Playwright 1.62.1: [capture 40142986865](https://github.com/e-kosten/pf2e-atlas/actions/runs/40142986865)
passed all 12 scenarios using the authenticated fixture from CI 40142708941,
merge commit `3137c52698ea14989b10dd955ec81c10f02afdd7` (tree identical to the
source head above). Binary SHA-256:
`9d70295cbf9983bad2a6d0be3b0ead2fa59aa4410f6f828857a9b7fc77817c5d`;
frontend index SHA-256:
`77a45203124dffbb6637e5f49a0ef219ca0d339a8cbd5a94f13142502801f942`.
Parent reviewed ten changed desktop/tablet pairs; ten other desktop/tablet frames
are byte-identical to the independently retained 751366ab capture review.
Independent Astra reviewed twelve phone pairs, including all four previously
excluded source/fallback frames; eight other phone frames reuse verified hashes
from the prior 751366ab approval. All 40 final Linux references are reviewed.
Candidate hashes
and evidence are under `scratch/review/creature-final-visuals/r24/linux` and
`scratch/review/creature-final-astra`.

The later Stunned rules link does not appear in these 40 checkpoint states. It has
separate actual Chromium/WebKit navigation, native-link and focus-return review;
final CI must compare its candidate against these references without updates.
The 16-case recovery matrix also checks keyboard Enter, result focus, innermost
Escape, canonical HP/receipts, base-action visibility and unchanged 1px anchor
tolerances on all four projects. These are browser/device emulation, not physical
device certification. Capture/review and ordinary comparison are separate evidence.

## Initial creature/encounter reference provenance

The macOS set captures the human-accepted r19 direction with the independently
reviewed shared Trait header repair and condition Edit-tooltip touch fix. Source
candidate: `7e7b1b5a945de0f104a1a42c2807ce1cc3601a51`, Playwright 1.62.1 on macOS
arm64, using the four maintained projects and isolated browser fixture. All 12
capture scenarios passed their interaction and geometry assertions. The damage
checkpoint waits for the authoritative Total and enabled Apply action, rather
than accepting an incidental updating state.

All 40 before/candidate pairs were viewed before adoption: parent reviewed the
20 desktop/tablet pairs; independent Astra reviewed the 20 phone pairs. Changes
preserve condition precedence, damage facts and source-removal fallback while
showing the accepted runtime lane, manual resource chips and contextual damage/
recovery rows. Editors no longer leave the initiating Edit tooltip above their
controls. Retained results and settled totals remain visible. Prior references
were preserved under `scratch/review/creature-final-visuals/before`; review details
and candidate hashes are under `scratch/review/creature-final-astra` and
`scratch/review/creature-final-visuals`. Capture does not establish a normal
comparison pass; the full four-project suite must compare these references with
snapshot updates disabled. Linux references are reconciled separately from the
authenticated hosted fixture; macOS PNGs must not be copied to Linux.

## Accepted Ant foundations

The current sets include the accepted shared pane spacing (16px rather than
14px) and persistent neutral cue on editable HP values. They preserve condition
facts, modal interiors, recovery controls and turn-reminder behavior. Each changed
frame was compared with its preserved reference before adoption; screenshot
creation alone does not establish acceptance. Exact `maxDiffPixels: 0` comparisons
remain enabled.

- macOS: 32 frames captured on `1d92e149` with Playwright 1.62.1, macOS arm64;
  eight remain byte-identical to the initial references below. Parent review
  covered 20 desktop/tablet pairs, independent Astra review covered 12 phone pairs.
  Two phone Chromium frames differ only by 21 and 3 pixels at one RGB level;
  their state and geometry are unchanged.
- Linux: 30 frames from [focused run 40017234470](https://github.com/e-kosten/pf2e-atlas/actions/runs/40017234470);
  ten remain byte-identical to the initial Linux references. Parent review covered
  20 desktop/tablet pairs; independent Astra review covered ten phone pairs.
  The authenticated fixture came from CI run 40008176093, merge commit
  `60c77188eab2fcfda71aa18d319af219e6d3d266` (PR head `e57e54e3`), with binary
  SHA-256 `fa517425268c7df0a9488f2a0828c7963eae38adbe9efbccf0b3ffc88a6104a1`
  and frontend index SHA-256
  `5ece17c1221895611c6cb27d52e158834aed132137dfbc1cc1ae6c01cd9f7b6c`.

Both captures passed their 12 interaction scenarios. Ordinary device runs must
subsequently compare the reviewed references without snapshot update. This
reference update does not accept a creature or encounter redesign.

## Initial macOS (darwin) provenance

The initial 40 files preserved the condition-system review evidence, copied byte-for-byte
from the retained implementation worktree's
`scratch/condition-system/refinement/visual`. The original 36 were captured at
09952c71f90ce9cf97a772512fee70371c4bbfd2; four Quickened references were added at
2374dc1d. All 40 matched the final PR91 candidate
09426bf38e6cb15bfb2667625a06a73a06c7f678, merged as
8ddba4630ac17d36078c7a6c7a2c0e82686e9c48. Playwright 1.62.1 on macOS arm64.

## Initial Linux provenance

The initial 40 references were captured and inspected from
[focused run 39901708040](https://github.com/e-kosten/pf2e-atlas/actions/runs/39901708040),
using the authenticated fixture from CI run 39901045751, source merge commit
f7ab2feaca62610b97c76114ec7d1adde8943b93 (PR head e223e9bd).
Ubuntu 24.04 x64, Playwright 1.62.1, the four projects in
`playwright.devices.config.ts`. The fixture's SHA-256 is
`2029e655226a5d50d84b85222c98dffbf4ce9609d5b11f2cb650035a8ce2eb40`;
its frontend index hash is
`b7f08c3f149f65cc5d7c0d09f1e1de284f5e14c7ec58a242d4560a0884dd76b4`.

Product Rust/frontend source and dependencies match merged PR91 (8ddba463).
System-font metrics and wrapping differ between Linux and macOS; Linux references
preserve the existing Linux rendering and do not replace the macOS references.
Review covered source precedence, removal fallback, retained recovery anchors,
pending/settled previews, Quickened restrictions and turn-reminder states. All 12
capture scenarios passed their interaction/geometry assertions. Capture alone
is not comparison evidence; normal CI must subsequently compare these files.
