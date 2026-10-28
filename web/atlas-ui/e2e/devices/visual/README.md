# Device visual references

Normal device runs compare exact pixels at ten condition-refinement checkpoints
per project, alongside interaction and transition assertions. References are
separated by OS and project; keep the pinned Playwright version, viewport, theme
and fixture state identical. See CONTRIBUTING.md for deliberate capture/review.

## Condition correction recovery and damage references

macOS arm64, Playwright 1.62.1, source
`f777261b2b8c4da9e7e3ba7e7007961b2dc60202`: four focused capture cases pass,
followed by an ordinary 68-case condition/form/reminder run with snapshot updates
disabled. Twelve frames change: `turn-damage`, `recovery-before` and
`recovery-cleared-anchor` in each maintained project. The damage checkpoint shows
Damage taken and its After IWR explanation. Recovery places Roll/Clear together,
wraps the action group within narrow viewports, and uses native centered top
placement so result growth preserves the popup's side of the anchor. Existing
position/input geometry tolerances remain unchanged; new assertions require both
actions fully in the viewport and the same popup side before/after resolution.
Parent and independent Astra reviewed the changed frames; independent follow-up
review accepted all eight final recovery frames after the phone Chromium side
flip was repaired. The other 28 macOS references remain intact. This is automated
browser/device emulation plus screenshot review, separate from the human's
accepted full-corpus functional preview.

Linux x64, Ubuntu 24.04, Playwright 1.62.1:
[capture 40758969195](https://github.com/e-kosten/pf2e-atlas/actions/runs/40758969195)
passed desktop/phone Chromium and tablet WebKit; phone WebKit stopped at the
pre-existing post-editor passive Edit-tooltip checkpoint before recovery.
[Phone WebKit capture 40759493166](https://github.com/e-kosten/pf2e-atlas/actions/runs/40759493166)
then passed all three repetitions on the same unmodified source and fixture.
These captures use CI40758209873's authenticated synthetic merge
`1c9d41add479bac11fde929c21c6a4e535c2a36a`, whose tree
`2a02f9bb6e2906270d0bdbcf4a7822039571804c` matches reviewed source `f777261b`.
Binary SHA-256:
`a2416969c74f8686959fad5106d340e911e60f6a3c3b72d423204923181ee3a8`;
frontend index SHA-256:
`5d9697feac8b4435c633ac20eeadc781eaff8bae5b926d9bb9770921e6204bdf`.
Parent and independent Astra reviewed the twelve changed Linux frames across all
four projects. Actions remain visible, damage guidance is readable and recovery
stays above its anchor. The existing phone WebKit page-scroll adjustment remains.
The other 28 Linux references stay intact, including incidental phone Chromium
fallback raster differences excluded from adoption. Ordinary final-head CI with
snapshot updates disabled remains required; capture is separate evidence.

The follow-up device harness gives the long precedence/recovery scenario its own
90-second budget, including cold browser setup; individual assertion timeouts and
pixel/geometry thresholds remain unchanged. An API-request-owned encounter fixture
performs bounded, asserted cleanup after page teardown. A cancellation probe
recorded DELETE200 after an intentional timeout and a successful following run
with the same encounter name.

The post-edit document checkpoint now deliberately focuses the updated 1d12
condition-name link and dismisses passive hints. This defines the screenshot state;
it does not certify automatic modal focus restoration. Four macOS
`precedence-sources` frames change only their focus cues. Parent and independent
Astra reviewed every pair; other frames retain their preceding references,
including excluded incidental 1/255 raster changes. All four capture cases and the subsequent twelve ordinary repetitions pass with
snapshot updates disabled. The product source remains `f777261b`; this follow-up changes the test harness and
reference focus state only.

Linux x64, Ubuntu 24.04, Playwright 1.62.1:
[capture 40767178596](https://github.com/e-kosten/pf2e-atlas/actions/runs/40767178596)
passed all four cases using CI40766561296's authenticated fixture. Synthetic merge
`50a9981d41ea930fda537ba5a5c2130057b96e2d` combines test-only main `ce3f6442`
and harness source `e958db26`. Its tree `c8fe5b15b80b276c8f4f3ebc0d4d47f48e8741af`
matches the explicit integration `72abb4a9`. Binary SHA-256:
`90e97ce64f185b5985928b9315b919a06a1c75e128bd06c4f04933eb539f5a4f`;
frontend index SHA-256:
`5a3c68c619e99e7363b10befa4bb40d846ccac72e734c33b07e2aa74ce31eaed`.
The integrated upstream checkpoint dismisses each passive tooltip layer before
comparison. Parent and independent Astra reviewed all four Linux source-focus
pairs: only the intended outline moves to the condition-name link, with no
layout, scroll or clipping changes. Only those four frames are adopted; the
other 36 references remain intact. The incidental phone Chromium fallback
edge-raster differences are excluded. Ordinary final-head CI remains required.

## Encounter settings and participant title alignment

macOS arm64, Playwright 1.62.1, product source
`97075725` (including UI commit `0070fcbf`): all 12 condition-refinement capture
cases pass. The 40 macOS references show the compact encounter cog, participant
settings cog and optically aligned title/classification. Parent reviewed desktop
diffs; independent Astra reviewed before/after pairs across all four projects and
two additional phone frames. Modal bounds, wrapped choices, recovery anchors and
source-list geometry remain stable. Sparse glyph-edge raster differences outside
the headers do not move controls or change content. The Linux references below
come from a separate Ubuntu capture; macOS pixels are not reused for Linux.
Ordinary comparison with updates disabled passes the 187 other device cases;
all four life-status cases pass after updating their participant menu interaction.

Linux x64, Ubuntu 24.04, Playwright 1.62.1:
[capture 40717852651](https://github.com/e-kosten/pf2e-atlas/actions/runs/40717852651)
passed all 12 focused interaction/geometry cases using CI40717173268's fixture.
The fixture source is merge `acc5cb5a74fa19f0d19c603e494e1a9bee65a4e5`, with parents
main `bb80d85f` and reviewed PR140 head `0f48210e`. Binary SHA-256:
`5ebf6522dd13fcc45ba380ca0d7dcacea26dbab6873ac6b6f115eed835b61658`;
frontend index SHA-256:
`e404bd492ce271195a7ae2dcf5ad9a085c09ab61b5868e958173e774f02d0b2b`.
All 40 Linux images retain their original dimensions; parent pixel review found
meaningful differences confined to the changed heading regions. Independent
Astra review compared one before/after pair per project and four additional
phone/tablet frames. Modal bounds, wrapping, recovery anchors, source-list spacing
and focus outlines remain stable. Ordinary CI comparison keeps updates disabled
and exact pixel tolerance zero.

## Intrinsic dice-popover recovery references

PR128's shared intrinsic sizing changes the recovery popover width and content
inset while retaining its right edge, arrow and controls. The eight Linux
`recovery-before` and `recovery-cleared-anchor` frames were reviewed by the parent
and independent Astra across all four projects. Other Linux references remain
unchanged; two incidental phone Chromium raster differences from capture were
excluded. Exact pixel tolerance remains zero.

Linux x64, Ubuntu 24.04, Playwright 1.62.1:
[capture 40457056559](https://github.com/e-kosten/pf2e-atlas/actions/runs/40457056559)
passed all four focused interaction/geometry cases using CI40453942587's fixture.
Its synthetic merge `51c342dc9b8889a1700bf2d63bc180a9d3fe8a7d` has the same tree
`ce5dd89d11d50e2f30020a025101bcfbc429449e` as reviewed PR head
`206b9829a26057ec3f0fa0833b7f86a3af00d750`. Binary SHA-256:
`9a5e8f8fd9a8453ca0256866e1ffac634b3325f4c3f88331607a1811b2eacb1c`;
frontend index SHA-256:
`1875c1ee930bffdd15c1365cbe5b3047916d023b0691e6e2bb9f472dccc562e2`.
All four before frames match the failing ordinary CI actuals byte-for-byte.
The existing macOS references are preserved; these Linux images were captured
separately. Ordinary hosted comparison with updates disabled remains required.

## Early reaction controls

macOS arm64, Playwright 1.62.1, source
`c68cf7795b191ab4fbec498ade4ef026f87aed0c`: all 12 condition-refinement cases
pass with snapshot updates disabled after adopting 24 reviewed frames. Six frames
per project now include the pre-turn `Reaction [Allow]` control and consequent
condition-list positions. Parent review covers desktop Chromium and tablet WebKit;
independent Astra review covers both phone projects. Phone WebKit also has small
viewport/modal-position shifts, with field layout, focus and reachable controls
preserved. Unrelated tiny border differences in three turn-damage frames were
discarded; those original references pass the ordinary comparison run.

Linux x64, Ubuntu 24.04, Playwright 1.62.1:
[capture 40421151327](https://github.com/e-kosten/pf2e-atlas/actions/runs/40421151327)
passes all 12 cases against CI 40420614895's fixture, synthetic merge
`fa90cff879538b4bc355e4c8c892f454309a86fd`. Its tree matches source `c68cf779`
(`33cce1b29310db995d338b81f593d5ffc52f924e`). Parent reviewed desktop/tablet;
independent Astra reviewed both phone projects. Adopted 22 product frames, keeping
two incidental border-only changes out of the update. Phone WebKit wraps Allow
onto a separate row and changes viewport/modal positions while preserving fields,
focus and reachable actions. Ordinary hosted comparison with updates disabled
remains required. Fixture SHA-256:
`2f3165ca4f80d4360a3cf72e6d3339ced123487fff832ad66ea10fab714ed8bf`;
frontend index SHA-256:
`2180e87aef2d9cc541b8f8809c35cf43ba2f28ddd956b4689e6047df3d84e555`.

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

### Accepted explanation cues

The quiet underline for explainable values replaces the older colored action-count
button. macOS arm64 / Playwright 1.62.1, source `10f1a0ca`, changed only the desktop
Quickened checkpoint. Parent and independent Astra reviewed its before/after/diff;
modal content, controls and geometry remain unchanged. The ordinary 135-case run
passed 134 cases, with only this expected reference difference; after adoption the
four-project focused Quickened run passes with snapshot updates disabled.

Linux x64 / Ubuntu 24.04 / Playwright 1.62.1:
[capture 40269847894](https://github.com/e-kosten/pf2e-atlas/actions/runs/40269847894)
passed all four focused Quickened cases using CI run 40269510944's artifact,
reviewed PR head `10f1a0cace07e48620a617058e724adb42f0f576` and synthetic merge
`928ea1819005b4975b001965c838a4e1b1fce4e0`. Binary SHA-256:
`5cd9752c8bce67d7e0a13eaa632b60b00ce54911f91b9709a49f95751d142d36`;
frontend index SHA-256:
`8ab47ebdca147f5af044daa303bca5fbc1679f9372dc8ea6decdab5f48fba420`.
Only desktop Chromium and phone Chromium Quickened PNGs changed. Parent and
independent Astra authenticated hashes and reviewed both pairs: desktop shows the
accepted cue plus a one-pixel dimmed-background alignment change; phone has 146
changed background pixels and none inside the modal. Phone/tablet WebKit remain
byte-identical. Pair evidence and hashes are in
`scratch/pr-heightening-review/linux-capture-40269847894`.
Ordinary hosted comparison with snapshot updates disabled remains required.

Current-main integration `318258e8` retains the newer HP workflow references below.
[Linux capture 40271798183](https://github.com/e-kosten/pf2e-atlas/actions/runs/40271798183)
passed four focused Quickened cases against CI40271508269's authenticated fixture,
synthetic merge `c318559a8d14d95b3f8970e929776c6c3a623ad0` (parents `d22a68f3`
and `318258e8`). Binary SHA-256:
`da64a3a8c388c696b6613f131bf988876a083c445878f1ef80ce2cda9f9848da`;
index SHA-256: `656fa09e98695af8d78492e3adda772978a75d41bcb12e825c2004fc410d8033`.
Parent and independent Astra authenticated all 40 pairs. Only phone Chromium
Quickened changes: 113 dimmed-background pixels, none inside the modal. Its current
SHA-256 is `e807aee4550e94786fc7d5721feab617958622a38fb9cce834e53c60afe951fb`.
This resolves the overlapping HP/underline reference; all other imported references
remain intact. Combined macOS ordinary matrix passes all 143 cases. Evidence:
`scratch/pr-heightening-review/linux-integration-40271798183`.

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
