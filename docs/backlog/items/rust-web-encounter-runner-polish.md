# Encounter runner usability follow-through

Status: proposed
Priority: soon
Owner: unassigned
Last reviewed: 2026-10-30

## Current baseline

PR61 delivered the compact roster, repeated Add workflow, intentional creature
facts, HP editing and condition-duration/reminder interactions. PR62 added spell
rank/resource presentation, and PR63 added initiative, attack/damage and roll-table
dice interactions. These are implemented starting points, not instructions to
repeat the earlier layout experiments.

Reference and encounter readers share family presentation; encounters add bounded
Rust runtime overlays and persisted participant state. General reference previews
retain context through the shared popover/drawer policy. Initiative rolls and HP
changes use backend-owned contracts; the UI must not duplicate arithmetic or
invent source mechanics from display strings.

## Reported narrow Actions explanation clipping

The [PR160](https://github.com/e-kosten/pf2e-atlas/pull/160) implementation handback
reports clipping in the encounter Actions calculation explanation at a 390px
viewport, while the DOM values were correct. Attribution to existing layout is
based on unchanged explanation/CSS owners, not a historical-browser reproduction.
The original screenshot was a temporary local review artifact; this backlog does
not depend on its availability. The documentation closeout did not reproduce it.

Before selecting a fix, reproduce the affected explanation in a current supported
narrow browser and identify the clipped content/container. Preserve the Rust
calculation values and shared popover/drawer policy. Treat this as a bounded
presentation follow-up, separate from numeric conversion or general layout redesign.

## Remaining work

Identify current problems through actual encounter use before choosing another UI
slice. Preserve current participant/HP/condition state, repeat-add behavior,
manual initiative editing, current-turn/selected-participant distinction, shared
preview controls and accessible responsive layouts.

The substantial known follow-ups have their own owners:

- [Party definitions](encounter-party-presets.md): define once and reuse across
  encounters; discuss party membership/storage before implementation.
- [Condition mechanics](rust-encounter-condition-mechanics.md): complete supported
  numerical and contextual effects without implying that every rule is automated.
- [Turn lifecycle](rust-encounter-turn-lifecycle-mutations.md): extend the existing
  duration/reminder events for further rule effects rather than creating browser
  counters or silently auto-rolling table decisions.
- [Capability follow-through](rust-integration-recovery.md): encounter status,
  action-budget policy, supported adjustments and hazard state have separate
  approval and implementation scopes.
- [Spell forms](rust-spell-rank-preview.md): alternate forms remain distinct from
  implemented rank preview and resource tracking.

Use earlier integration/AO work as feature inspiration, not correctness evidence.
Discuss material UX changes first, provide an early functional checkpoint, and
complete Astra browser review and final tests after the direction is accepted.
Keep existing Ant controls and shared family renderers; do not introduce another
stat block or feature-local overlay design.

The [2026-10-27 loading audit](../../research/ui/2026-10-27-loading-continuity-audit.md)
identifies remaining immediate mutation-button busy indicators and late-arriving
lifecycle/status controls. Preserve immediate input guards while delaying visual
progress; review the initial control footprint without inventing unavailable actions.
