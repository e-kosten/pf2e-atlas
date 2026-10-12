# Rust Web Encounter Runner Polish

Status: in_progress
Priority: soon
Owner: unassigned
Last reviewed: 2026-10-11

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
