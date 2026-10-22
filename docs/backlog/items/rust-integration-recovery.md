# Source-modeling Capability Follow-through

Status: in_progress
Priority: CF-10 expanded Boolean reference filters and CLI parity (third PR)
Owner: item-specific; CF-09/10 in browse-reference-search, CF-07 independently owned
Last reviewed: 2026-10-22

## Purpose and authorization

Preserve useful capabilities from main and the earlier integration/AO pathways
within the current source-backed architecture. The user accepted the ten-item
sequence below and requested persistent tracking. This approves the sequence and
its documentation, **not implementation of every recommendation**. Confirm the
selected scope and unresolved product behavior before implementation. Do not
interpret this list as ten mandatory merge blockers.

CF-01/02/03 merged in PRs #68/#69/#71; CF-04/05 merged in PR #73.
All are now on main through integration PR #25 (41865f60). The user selected
CF-06 + CF-08 as the next combined slice targeting main. The user approved the product policy, including consistent creature/hazard
controls and subdued disabled-hazard initiative facts. CF-07 remains with the
condition workstream. The 2026-10-21 handoff identifies CF-06/08 as already merged
in PR #78; their older review rows below are historical evidence. CF-09/10 product
design and implementation are approved as the three-PR sequence documented below.
Mixed-failure precedence is runtime (3), invalid input (2), domain
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
| CF-01 | CLI batches and error contracts | merged | Approved 2026-10-18 | [PR #68](https://github.com/e-kosten/pf2e-atlas/pull/68); merged as b6a0a684; local validation passed |
| CF-02 | Creature/hazard reference facts and activity details | merged | Approved 2026-10-18 | [PR #69](https://github.com/e-kosten/pf2e-atlas/pull/69); merged as b0ac0ae5; user accepted reference layout; local validation passed |
| CF-03 | Ritual/spell, feat/effect and journal/table reference completeness | merged | Approved 2026-10-18 | [PR #71](https://github.com/e-kosten/pf2e-atlas/pull/71), f950b54f; UI accepted; local gates passed |
| CF-04 | Consumable presentation and nested-spell navigation | merged | Approved together with CF-05 | Locally complete; accepted preview; Rust/UI gates and independent browser review pass; [PR #73](https://github.com/e-kosten/pf2e-atlas/pull/73); user accepted preview |
| CF-05 | Physical query fields and labels | merged | Approved together with CF-04 | Locally complete; typed filters, migration and exact labels validated; [PR #73](https://github.com/e-kosten/pf2e-atlas/pull/73); user accepted preview |
| CF-06 | Encounter status and action-budget policy | in_progress | Approved | Based on main 41865f60; locally complete; user accepted preview; Rust/UI gates, 20 device journeys and independent Sol browser review pass; [PR #78](https://github.com/e-kosten/pf2e-atlas/pull/78) |
| CF-07 | Supported condition/spell adjustments and casting reminders | planned | Pending | Not started |
| CF-08 | Hazard Active/Disabled state | in_progress | Approved with CF-06 | locally complete; single Active/Disabled state; user accepted preview; Rust/UI gates and browser reviews pass; [PR #78](https://github.com/e-kosten/pf2e-atlas/pull/78) |
| CF-09 | Browse sorting | merged | Approved 2026-10-21 | [PR #98](https://github.com/e-kosten/pf2e-atlas/pull/98), merged as 09d2bec5; human UI acceptance, local gates, independent reviews and exact-head CI passed |
| CF-10 | Relationship-scoped full results and filtering | in_progress | Expanded scope approved 2026-10-22 | Core [PR #97](https://github.com/e-kosten/pf2e-atlas/pull/97) merged as a1601f87 with acceptance/gates/CI; third PR on `feat/relationship-filter-picker` now includes shared Boolean predicates, Advanced integration and CLI parity; initial picker preview retained at port 4940 |

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

Validation follow-up: generated source contracts are pinned to PF2e 6.12.4,
while the vendored source used for the Aasimar Redeemer sample is 7.12.2. Bolts is
`type: consumable`, category `ammo` at the pinned commit, but `type: ammo` in 7.12.2.
The active manifest is `system.pf2e.json`; the retained `static/system.json`
reports an older 7.9.1 and must not identify this corpus. The latter representation falls outside the admitted union and makes the whole actor `items`
collection invalid before presentation. Decide source-version support/enforcement
and per-item failure isolation separately; this is not evidence that the pinned
6.12.4 ammo representation is unsupported. Do not drop an invalid item silently
or synthesize a focus entry from resources. CF-02 preserves the unavailable state.

User checkpoint: discuss the source-pin mismatch further before proceeding into
phases 2 and 3. Do not start remediation now. Discussion must cover intended
upstream support, source/model version enforcement, scope of affected records,
and whether invalid embedded entries should isolate failure to that entry.
Do not infer that this checkpoint authorizes a model upgrade or admission redesign.

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
content terminology consistently across CLI, API and UI: `embedded_path` for
checked navigation, `embedded` for child links and `embedded-reading` for batches. This terminology work is deferred from CF-01.

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
Keep per-item GP semantics and make package pricing clear. This slice uses exact
authored Usage values and pinned readable labels; no normalized hands axis.

Acceptance: healing vitality differs from vitality damage; root/immediate-embedded
filters use the same typed extraction; unknown values retain query semantics;
attachment to a two-handed weapon is not classified as two hands to hold the item.
**Consumable base-item filtering is deprioritized by user decision**, not part of
the default implementation scope for this item.

CF-04/05 local validation: Rust verification gate; frontend verification (686
passing tests); 20 Chromium/WebKit responsive journeys; independent Sol browser
review at 1200/800/390px, including touch, native full-page links and canonical
original navigation. The shared request conversion now explicitly converts a
navigation field into the request field list. The nine-record model-free preview
supports bounded reference acceptance, not full-corpus or semantic-search release
acceptance. The user accepted the final preview and authorized publication of
`feat/cf04-cf05-consumable-reference` against `integration/source-modeling`.

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

Hazards use Active/Disabled instead of Active/Defeated, with HP tracked separately.
Damage reaching zero disables; healing does not reactivate. Existing defeated
hazards migrate to disabled. Keep disabled
hazards readable/editable and exclude them from automatic turn selection.
Reactivation is explicit; authored Reset instructions remain reference material.

Acceptance: disabling an undamaged complex hazard preserves HP and other state,
skips its turn, and does not erase it; reactivation does not heal or execute rules.
A broad hazard state machine and participant baseline reset are not implied.

### CF-09 — Browse sorting

Approved 2026-10-21 as the second PR in this workstream. Without semantic search,
sort the entire eligible filtered set; with semantic search or Similar, sort the
entire bounded matching set before pagination, keeping its membership independent
of the selected order/page. Preserve existing retrieval policy and label bounded
coverage honestly; never switch to FTS-only matching because the user chose a sort.
Explicitly selected attribute order survives adding/clearing semantic criteria;
fresh text/Similar searches default to Relevance. Missing/null/invalid/inapplicable
values sort last in both directions, with normalized-name/canonical-key ties.
Saved-list attribute and Random order is temporary and never writes member positions.
Order is URL state; criteria/order/page-size changes reset paging without losing
other filters or drafts. Arbitrary key order and semantic threshold redesign stay
outside scope. Keep this implementation separate from the first relationship PR.

Restore useful name, level/rank and unit-price ordering through shared backend
requests, with deterministic tie-breaks and an explicit unavailable-value policy.
Integrate pagination, URL/search identity, saved-list scope and generation checks.
Keep text/Similar relevance behavior explicit.

Approved UX refinement after the initial preview: a visibly labelled Sort control
opens a shared popover. Attribute rules have explicit priority, field and direction;
later rules break ties, with one rule per field and accessible add/remove/reorder.
Relevance, Random and list-only Saved order are alternative complete modes.
Random orders the entire same eligible/admitted set using a generated seed;
Shuffle or an explicit seed edit changes it and resets page1. Paging, Back and
reload preserve the seed through URL state. Seed editing/copying stays inside the
popover. Both attribute and Random modes preserve the existing count/coverage
meaning, generation checks, retained results and dirty drafts.

Acceptance: sorting applies to the entire eligible set before pagination; later
pages do not merely reorder the first candidate window; changing sort resets
paging coherently and preserves the intended filters. Multiple rules resolve
ties in the displayed priority order. Same-seed Random concatenated pages match
the whole-set order; Shuffle changes it without changing membership or saved
positions. Sort is discoverable and all controls remain reachable on phones.

### CF-10 — Relationship-scoped full results and filtering

Approved 2026-10-21. First PR: record-origin Links to/Linked from constraints,
full Related escapes, and the Similar search-entry workflow. Relationship criteria
remain visible at the top level with ordinary filters and compose with text/
Similar before ranking. Main entry only / Whole entry controls source-side references
in both directions; the destination remains exact. New whole-record actions include
contents by default; Related entry preserves its effective selected scope, and
embedded selections never broaden to siblings/parents. Each displayed result is
one distinct root, with checked occurrence provenance and independently bounded/
pageable evidence. Keep existing bounded Related/encounter readers intact.

Relationship UI displays human names, never record IDs, including unavailable
states. The active criterion's popover switches Links to / Linked from while
retaining the exact selected entry, contents scope and other search criteria.
Disambiguation uses kind/publication/parent names. Similar occupies the
main search control as `[Similar to] [Leather Armor] [Clear]`; the name is
selectable/copyable and read-only. Clear retains other filters and returns focus
to keyword entry. Keyword and Similar modes share search/clear affordances and
control dimensions. Relationship actions live inside Related; result reference
evidence shares the Matches button/popover, with human source-to-target labels
and independent occurrence paging. No parallel editable search field is shown
for an active seed.

Third PR, expanded scope approved 2026-10-22 after the initial picker preview:
make reference conditions first-class leaves in the existing typed predicate, with
AND/OR/NOT alongside ordinary filters. Remove the separate AND-only collection and
all alternate request/eligibility paths. Links with a chevron is a toolbar shortcut;
Advanced can add/edit/group/negate the same conditions. Simple conjunctions retain
chips; complex expressions show grouping without implying a flat AND. Clear filters
clears the complete predicate while retaining text/Similar/order. Record-origin
actions use the same model and conjoin with the entire active Search expression.

Reuse the shared list/encounter finder independently of parent criteria. Choose one
root per confirmation, default Links to / Whole entry; preserve existing embedded
selections until explicit root choice. Direction/scope remain per condition. Only
safe direct-conjunction duplicates are suppressed. Exact old separate-parameter
URLs are explicit unsupported-format errors, never silently broadened results.

CLI browse/text/Similar, contextual counts/values and independently paged occurrence
evidence use the same model and workflow. Canonical CEL link functions support full
Boolean composition and exact checked selections; strict-name convenience flags add
conjunctive root conditions without inheriting candidate scope. Print-filter remains
artifact-free. Search JSON carries full rows/clause evidence rather than summary-only
payloads; update help/examples and the bundled agent skill. No installed skill edits.

Every target is checked even under NOT/unreachable branches before inference. Root
counts remain distinct. Positive evidence belongs to successful Boolean proofs;
negative absence and unmatched branches have no fabricated occurrences. Clause IDs
replace positions in a separate array. Preserve generation/fingerprint checks,
self-exclusion and honest unavailable suggestion contexts. Reference leaves apply to
candidate roots; authored Exists correlation and multi-hop traversal stay outside
scope, as do inferred possession and condition-infliction filters. No new source pin,
artifact schema or rebuild is implied. See ADR0057 for the shared ownership contract.

Keep lightweight Related previews, with an escape to full incoming/outgoing
results that supports ordinary filters and paging. Preserve exact owner versus
root scope, occurrence provenance, distinct-root counts and truthful truncation.
Use one shared backend relationship contract, not a client-side list of the first
100 linked keys or a second query engine.

Acceptance: high-degree references can be browsed beyond the preview cap and
filtered to a kind; repeated occurrences do not inflate root counts; selected-owner
queries do not silently include sibling links; eligibility precedes ranking/paging.
All contextual filter counts and value suggestions reflect the relationship scope,
including intersections with saved-list membership. Self-exclusion preserves the
relationship constraint when replacing an ordinary filter option; counts remain
distinct-root counts across the complete eligible set. Scope changes invalidate
previous counts. Existing text/Similar count availability rules remain unchanged.

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

### Approved source-version follow-ups

CF-02 investigation found model pin 6.12.4 versus local data 7.12.2. Full admission
census: 203 unsupported ammo roots and 712 actors with unavailable items collections
in 7.12.2. Keep these follow-ups distinct from CF-03 through CF-10:

- SV-01 (merged; [PR #70](https://github.com/e-kosten/pf2e-atlas/pull/70), locally validated): one canonical immutable revision for default
  reference acquisition and generated model identity. Preserve supplied checkouts
  and saved state; validate source compatibility before product builds. Initial
  supported revision remains 6.12.4. ADR0052 defines the boundary.
- SV-02 (deferred by user 2026-10-18 after approved bounded investigation): release 8.6.1 at
  28d9d156f252d084371d994dc359509b49eadf18 needs manifest/declaration/provenance
  tooling updates. Isolated probes achieved complete extraction, but generation
  stops on changed authored weapon and spell-overlay policies. This exceeds a
  simple pin/regenerate upgrade; downstream compilation/admission work is unmeasured.
  Schedule a separate upgrade review before implementation. No 8.6.1 model changes
  are included in SV-01.

Embedded-document failure isolation is a separate unapproved policy decision.
