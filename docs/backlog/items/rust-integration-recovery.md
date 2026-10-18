# Source-modeling Capability Follow-through

Status: in_progress
Priority: agreed sequence; CF-01 implementation approved
Owner: unassigned
Last reviewed: 2026-10-18

## Purpose and authorization

Preserve useful capabilities from main and the earlier integration/AO pathways
within the current source-backed architecture. The user accepted the ten-item
sequence below and requested persistent tracking. This approves the sequence and
its documentation, **not implementation of every recommendation**. Confirm the
selected scope and unresolved product behavior before implementation. Do not
interpret this list as ten mandatory merge blockers.

CF-01 implementation is now approved by the user; CF-02 through CF-10 remain
unapproved. Mixed-failure precedence is runtime (3), invalid input (2), domain
miss/ambiguity (1), success (0).

The user agreed to deprioritize consumable base-item filtering: only one of
1,669 consumables in the inspected PF2e 6.12.4 equipment pack populated
`baseItem` (Blindpepper Bomb, `alchemical-bomb`). This is source coverage evidence,
not a measurement of user demand. It does not retire base-item detail access,
weapon/armor filtering, or independently designed family discovery.

## Ordered work tracker

IDs remain stable as work progresses. The order is the agreed discussion/delivery
sequence, not a requirement to create a stacked chain of dependent PRs. Choose
bases from the current workstream and actual dependencies. A scope can be split
if review warrants it; preserve its ID and record every resulting PR here.

| ID | Intended PR scope | Status | Implementation approval | PR / candidate / validation |
| --- | --- | --- | --- | --- |
| CF-01 | CLI batches and error contracts | in_progress | Approved 2026-10-18 | Batch and record-response layouts accepted; full Rust gate passed; PR publication underway |
| CF-02 | Creature/hazard reference facts and activity details | planned | Pending | Not started |
| CF-03 | Ritual/spell, feat/effect and journal/table reference completeness | planned | Pending | Not started |
| CF-04 | Consumable presentation and nested-spell navigation | planned | Pending | Not started |
| CF-05 | Physical query fields and labels | planned | Pending | Not started |
| CF-06 | Encounter status and action-budget policy | planned | Pending | Not started |
| CF-07 | Supported condition/spell adjustments and casting reminders | planned | Pending | Not started |
| CF-08 | Hazard Active/Disabled state | planned | Pending | Not started |
| CF-09 | Browse sorting | planned | Pending | Not started |
| CF-10 | Relationship-scoped full results and filtering | planned | Pending | Not started |

### CF-01 — CLI batches and error contracts

Retain successful batch results alongside per-input failures in one JSON response,
with input order/identity and overall ok/partial/error status. No aggregate counts
or redundant partial flag. Preserve exit classes:
0 success, 1 domain miss/ambiguity, 2 invalid input, 3 runtime/environment failure.
Mixed failures use the highest exit class. Grammar/workspace admission failures
are whole-command errors; runtime interruption preserves earlier results and
marks later inputs unprocessed.

Scope extension: clarify CLI record output with selected `identity`, optional
`parent` for embedded selections, exact `selected` navigation and `content`. Reuse
shared semantic facts and remove the duplicate root summary. Preserve customized
embedded identity and effective rank-preview facts. Owned terminology remains CF-04.

Acceptance: valid + missing retains the valid result; two misses emit one valid
JSON document; malformed key, missing record and unavailable workspace are
machine-distinguishable. Use current DTOs, not legacy payload adapters.

### CF-02 — Creature/hazard reference facts and activity details

Select meaningful defense notes, size/hardness, contextual resources/ritual DC,
activity frequencies, self-effect links and strike-effect labels for shared
CLI/web/encounter reading. Preserve authored qualifiers without interpreting
prose into automatic rules. Do not restore default-zero legacy shield panels or
invent dedicated activity requirements/cost/uses fields from descriptions.

Acceptance: Gray Master retains void healing and ability frequencies; Night Hag
retains conditional save notes and Abyssal Plague. Optional blank facts remain
quiet; meaningful source values and unknowns are not confused.

### CF-03 — Ritual/spell, feat/effect and journal/table reference completeness

Recover ritual primary/secondary checks and caster counts, counteraction,
prerequisites and meaningful effect duration. Agree journal heading/availability
behavior and clear table-reference semantics. Keep technical rule-element dumps
in source inspection; alternate-form execution and media viewers are separate.

Acceptance: a ritual can be read without losing required checks/casters; feat
prerequisites and effect expiry are visible; unavailable pages differ from no
pages; table gaps/invalid ranges remain honest without implying saved deck state.

### CF-04 — Consumable presentation and nested-spell navigation

Terminology decision: replace product-facing owned/owners wording with embedded
content terminology consistently across CLI, API and UI; exact field names remain
to be finalized in this slice. This terminology work is deferred from CF-01.

Choose compact intrinsic facts (category, damage/healing, useful uses/capacity,
package pricing and secondary properties) and distinguish source-carried state
from live inventory. Expose the exact spell inside a standalone or actor-owned
consumable through the existing checked owner navigation. This is a missing
consumable-reader link, not a replacement for functioning actor spell navigation.

Acceptance: locally customized spell prose/damage survives opening the child;
Arboreal Wand's embedded Heal preserves base and contextual casting ranks; a
scroll shell with no spell gets no fabricated link. Quantity, charges and price
per item remain distinct. Verify the suspected non-Actor rank-context gap rather
than treating the earlier static observation as a reproduced defect.

### CF-05 — Physical query fields and labels

Prioritize useful typed consumable damage/healing kind/type and clear usage
choices. Agree a reviewed hands mapping before adding a normalized hands axis;
do not restore substring heuristics or confuse hands held with hands required.
Keep per-item GP semantics and make package pricing clear.

Acceptance: healing vitality differs from vitality damage; root/immediate-owned
filters use the same typed extraction; unknown values retain query semantics;
attachment to a two-handed weapon is not classified as two hands to hold the item.
**Consumable base-item filtering is deprioritized by user decision**, not part of
the default implementation scope for this item.

### CF-06 — Encounter status and action-budget policy

First agree the meaning of Defeated, ordinary casting versus retrospective
bookkeeping, current capability versus next-regain forecast, and numeric versus
duration Stunned. Choose a declared supported rule/source policy and transparent
Quickened/Slowed/Stunned composition. Do not copy old arithmetic as authority or
expand implicitly into a complete action-spending engine.

Acceptance: reads never consume conditions; duration stun never defaults to
numeric 1; totals and explanations agree; inactive status and any explicit
correction path behave consistently in Rust actions and UI controls.

### CF-07 — Supported condition/spell adjustments and casting reminders

After the relevant CF-06 policy decisions, select bounded Fatigued and
spell-attack/DC overlays, Stupefied check reminders, and supported elite/weak
spell-damage adjustments. Keep authored/effective values distinct and explain
unsupported cases beside the affected fact. Recording an attempt must not imply
spell success or automatic effect execution.

Acceptance: status penalties stack/suppress correctly with attribution; known
spell facts are adjusted once; unknown facts stay unknown; a casting reminder
does not incorrectly prevent recording spent resources for a disrupted attempt.
Damage classification/component selection follows the chosen source policy.

### CF-08 — Hazard Active/Disabled state

Agree a separate hazard-disabled state independent of HP/defeated. Keep disabled
hazards readable/editable and exclude them from automatic turn selection.
Reactivation is explicit; authored Reset instructions remain reference material.

Acceptance: disabling an undamaged complex hazard preserves HP and other state,
skips its turn, and does not erase it; reactivation does not heal or execute rules.
A broad hazard state machine and participant baseline reset are not implied.

### CF-09 — Browse sorting

Restore useful name, level/rank and unit-price ordering through shared backend
requests, with deterministic tie-breaks and an explicit unavailable-value policy.
Integrate pagination, URL/search identity, saved-list scope and generation checks.
Keep text/Similar relevance behavior explicit. Seeded random and technical key
ordering remain separate choices rather than automatic scope additions.

Acceptance: sorting applies to the entire eligible set before pagination; later
pages do not merely reorder the first candidate window; changing sort resets
paging coherently and preserves the intended filters.

### CF-10 — Relationship-scoped full results and filtering

Keep lightweight Related previews, with an escape to full incoming/outgoing
results that supports ordinary filters and paging. Preserve exact owner versus
root scope, occurrence provenance, distinct-root counts and truthful truncation.
Use one shared backend relationship contract, not a client-side list of the first
100 linked keys or a second query engine.

Acceptance: high-degree references can be browsed beyond the preview cap and
filtered to a kind; repeated occurrences do not inflate root counts; selected-owner
queries do not silently include sibling links; eligibility precedes ranking/paging.

## Tracking and completion discipline

- Before starting an item, record the selected behavior, authorization and any
  deliberate omissions. Revalidate against the current integration tip; the audit
  candidate below is historical, not evidence for later changes.
- Reuse this tracker and the task's current plan. Coordinate overlapping work
  with existing [CLI](rust-cli-skill-capability-follow-through.md),
  [detail](rust-web-record-detail-polish.md),
  [filters](rust-web-filter-ux-expansion.md) and
  [encounter](rust-web-encounter-runner-polish.md) items; do not maintain competing
  implementation checklists.
- Record branch/PR links, exact reviewed candidate, relevant test/interaction
  evidence and pending limitations. Local checks, human acceptance, hosted CI and
  merge are distinct states; a green test run alone does not complete an item.
- Use the backlog status vocabulary while open. When completed or deliberately
  retired, move that item's scope/decision/evidence to backlog history, replacing
  its open row with a history link in a completed-sequence ledger. Preserve stable
  IDs/order so remaining work stays navigable. Do not mark unapproved deferral as
  completed implementation.
- Update affected architecture guidance with any approved durable boundary change.
  Keep the source-backed end state; no old storage models or compatibility shims.

## Separate conditional or deferred work

Alternate spell forms remain in [Spell form preview](rust-spell-rank-preview.md).
Participant reset/replay, variant-family discovery, copy comparison, container/book
reading, stateful inventory/decks and any valuable legacy-state import require
separate scope decisions. Ranking changes remain in the existing
[search-quality evaluation](rust-search-quality-tuning.md). Removed raw-rule
panels, ranking knobs, synthetic affliction identities, generic metric authority
and macro product access are not implicitly scheduled for restoration.

## Analysis provenance

Audit candidate: `3825ef2b580d0ff0c4c161f93a3bb31a38a5db8c`.
Compared endpoints:

- main: `c7b74cbdc7c4df72b7c9a1c88c4f8dedbca959e7`
- simple physical families: `1ef0ab8fd13abf02e809a8ff47dd0ee9c3f97b29`
- consumables local/remote: `2042f619808e223927b1b35e1032ea8c4c0c1f7c` /
  `fa375724e8475f3fd42c7798783cd567039a5194`
- journals/roll tables: `cb4ebb61ae78198160503976ba2b154c6c884ec2`
- record-refactor local/remote: `997dd35b0772c25f2496c8584d313c8c979f9120` /
  `66d47fa9608c5f8c0b77e44efc85faade78d2b89`

Primary census: PF2e 6.12.4, `03942c8abd0f8617d4f0446d07bd8006520324fd`.
Detailed local evidence is retained in the audit worktree
`.worktrees/source-modeling-parity-audit/scratch/parity-audit/`:
`REPORT.md`, `PRODUCT-APPROACH.md`, logs and probes. Those ignored local artifacts
are not assumed available in another clone; this tracked item is self-contained
for scope and acceptance. Preserve needed evidence before retiring that checkout.
The older [recovery inventory](../../research/integration-recovery-inventory.md)
remains historical context. Current ownership follows the
[architecture overview](../../architecture/overview.md),
[ADR0046](../../architecture/decisions/0046-source-backed-reference-artifact.md)
and [ADR0047](../../architecture/decisions/0047-semantic-record-presentation.md).
