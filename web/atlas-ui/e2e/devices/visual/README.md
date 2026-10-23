# Device visual references

Normal device runs compare exact pixels at ten condition-refinement checkpoints
per project, alongside interaction and transition assertions. References are
separated by OS and project; keep the pinned Playwright version, viewport, theme
and fixture state identical. See CONTRIBUTING.md for deliberate capture/review.

## macOS (darwin)

These 40 files preserve the condition-system review evidence, copied byte-for-byte
from the retained implementation worktree's
`scratch/condition-system/refinement/visual`. The original 36 were captured at
09952c71f90ce9cf97a772512fee70371c4bbfd2; four Quickened references were added at
2374dc1d. All 40 matched the final PR91 candidate
09426bf38e6cb15bfb2667625a06a73a06c7f678, merged as
8ddba4630ac17d36078c7a6c7a2c0e82686e9c48. Playwright 1.62.1 on macOS arm64.

## Linux

These 40 references were captured and inspected from
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
