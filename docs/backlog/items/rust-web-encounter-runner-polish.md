# Rust Web Encounter Runner Polish

Status: in_progress
Priority: soon
Owner: unassigned
Last reviewed: 2026-10-13

## Problem

The encounter participant pane prioritizes general editing and maximum-HP
configuration over current combat state and actions. The roster truncates useful
participant names, while defenses and abilities require substantial scrolling.

## Desired Outcome

- Make roster names, initiative, current-turn state and HP easy to scan.
- Put current HP, damage/heal and conditions before occasional editing controls.
- Keep participant edits and HP overrides/provenance in compact disclosures.
- Use shared family presentation for defenses, movement and actions, with useful
  pane-responsive grouping and inline owned ability content.
- Keep source prose available without displacing immediate combat information.
- Review actual browser output and interactions independently with Astra.

Use the earlier integration branch as feature/layout inspiration. Validate
mechanical correctness against current Rust policies and source evidence.

## Current Implementation Scope

Shared family presentation supplies grouped combat mechanics. Existing activity
buttons open native quick lookup containing only requested authored prose and
current exact participant activity facts/explanations. Reference-content caches
retain no adjusted facts; participant, generation and activity changes remain
explicit without baseline child-detail fallback. Native full reference links and
nested source-dialog Escape/focus remain available. Content-free hydration
captures source generation once per deduplicated retrieval; manual-only encounters
remain usable without opening an artifact. The runner prioritizes HP, damage/heal and conditions, with
initially closed Edit details and HP details disclosures. Roster names wrap and
current turn has an explicit label. Existing participant state and edit-intent
semantics remain unchanged. Source content is available through explicit loading.
Immediate defenses, movement and actions share the useful combat view; secondary
skills, communication and runtime facts remain available below actions.

Focused tests cover keyboard controls, disclosure reset across participants,
HP/override intent and saved-response reload. Integrated gates, actual browser
state persistence and independent Astra visual review are required before the
candidate is offered for product feedback.

## Constraints

- Preserve existing persisted participant state and mutation semantics.
- Reuse shared family renderers and Ant controls; do not create another stat block
  or a feature-local family implementation.
- Keep adjustment, HP and condition arithmetic in Rust.
- Separate restoration of genuinely missing runtime behavior from layout repairs.
- Validate on disposable local state rather than modifying user encounters.

## Follow-up from encounter use

Review the encounter workflow as a whole before implementing these reported
issues. The current shared reference layouts are useful ingredients, not proof
that the runner's information hierarchy or interaction design is complete.

- Add several creatures without repeatedly reopening the picker. Quantity already
  supports copies of one selection; the current picker closes after each successful
  add. Compare keeping it open for successive additions with batch selection,
  retaining useful previews, quantities and clear mutation/failure feedback.
- Make the roster substantially more compact. Preserve readable names, initiative,
  current turn and HP while reducing repeated labels, multi-line controls and
  per-row actions. Evaluate a realistic larger encounter on laptop and wide screens.
- Review HP controls, HP details and participant Edit details together. Put frequent
  combat changes first and occasional setup/override controls where their purpose
  is clear. Preserve existing HP intent and persistence semantics.
- Align the creature classification and variant controls in the selected header,
  including wrapping and narrow-pane behavior.
- Clarify the top-right Edit action. The page action edits encounter metadata and
  lifecycle; the creature pencil edits a participant note. Distinguish both from
  participant Edit details through placement, labels and discoverable context.
- Replace the vague More creature facts disclosure with a deliberate secondary
  information arrangement. It currently contains perception details, languages,
  ability modifiers, skills and runtime notes. Reproduce the reported duplication
  across primary, secondary, adjustment explanations and source-reference views;
  do not assume those are all the same data. Avoid unnecessary repeated reading.

Reusable party definitions are tracked in [party presets](encounter-party-presets.md).
Creature initiative rolling and general dice interaction are tracked in
[product actions](foundry-inspired-product-actions.md); they require behavior
contracts as well as presentation work, rather than a cosmetic button addition.

Discuss the proposed end-to-end interaction with the user before UI changes.
Use the existing shared Ant controls and family renderers, and have Astra review
real visual output and interactions before presenting an implementation for review.

## Known Accessibility Follow-up

Opening and canceling Add PC, then opening Edit encounter, leaves two inputs
with the DOM ID `name`: the hidden PC form and the visible editor. Clicking the
visible Name label does not focus its textbox. These unnamed Ant forms predate
the mutable-workflow changes. Give form instances distinct namespaces through
Ant's Form API, and verify label-click focus with both dialogs previously opened.
Keep the existing field names and draft/session behavior.

## Related

- [Semantic record presentation](../../architecture/decisions/0047-semantic-record-presentation.md)
- [Rust web record detail polish](rust-web-record-detail-polish.md)
- [Rust encounter runtime mechanics surfaces](rust-encounter-runtime-mechanics-surfaces.md)
