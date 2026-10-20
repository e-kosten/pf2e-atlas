# ADR 0055: Shared Encounter Condition Evaluation

Status: accepted.
Date: 2026-10-19

## Context

Condition effects span displayed statistics, strikes, spells, senses, turn review,
HP and recovery. Implementing each surface independently creates contradictory
numbers and allows actions to bypass restrictions. Some effects are derived;
others change saved state once. Treating both as display adjustments repeats
consequences on reads or loses independent sources.

## Decision

`atlas-app-service::encounters::rules` owns condition policy. Its source adapter
reads checked `atlas-record` views into internal typed facts. Statistic targets,
abilities, causes, modifiers and action judgments are domain facts. Outgoing
presentation DTOs are not inputs to calculation. Display and mutation consumers
use the same evaluated facts; presentation translates them without rule arithmetic.

Independent saved condition sources retain identity, scope, parameters, duration
and origin. Resolve scope before suppression and derive maintained grants without
saving duplicate child rows. Removing one source cannot remove an independent
effect. Persistent consequences, such as Drained HP loss and falling Prone, use
explicit event plans. Correcting an entry differs from applying an effect now.
Behavioral subjects, such as a fascination or threat, are distinct from the source
that applied the condition. Missing subjects require context only when an action
depends on them; the causal source is not an implicit substitute.

Unknown facts stay unknown. An unavailable calculation does not silently use an
authored subtotal, assume a missing trait is false, or substitute an unheightened
spell. Ask only for facts relevant to the requested operation. Browsing a creature
does not require selecting a target or supplying geometry. UI action availability
and required inputs come from Rust projections; the browser does not switch on
condition names to implement policy.

`atlas-local-state` owns atomic event writes, receipts and pending consequences. A write recovers an existing receipt before checking revisions,
then validates the current snapshot before rolling or changing state. The same
request cannot repeat HP loss, resource expenditure, checks or turn processing.
Accepted boundaries also have unique participant/round/scope claims, so changing
the request ID cannot repeat the boundary. Reads and manual turn selection are
not lifecycle events.

A direct attack roll retains backend numerical adjustments and source/receipt
guards. Optional Target & checks assistance is read-only, stays beside the original
roll and cannot reroll or rewrite it. The spell popover reads condition reminders
and records a direct guarded use through the existing resource mutation owner.
Recording use asserts expenditure, not targeting success, completed checks or
permission to act. Reference flat checks are dice for the GM, not a saved action
state machine. Retch has a focused preview for the judgments its lifecycle action
actually needs. Its fixed action traits are not editable judgments; independent
source-specific answers are evaluated once and shared by preview and commit.
The action evaluator covers the live Strike, Cast and Retch workflows. Actions
the product cannot initiate do not have query branches.
There is no general action-query API, durable attempt history,
frozen action context or alternative cost-commit path.

HP events distinguish resolved damage, resolved healing, direct loss and exact
correction. Already mitigated table amounts never receive defenses a second time.
Saved maximum HP remains baseline capacity; condition-adjusted capacity is derived.
Track Dying and mortality are distinct from Defeated/Disabled participation markers.
Explicit level overrides and death thresholds are saved participant facts, not
values inferred from notes or transient form state.

Incomplete persistent damage is a typed reminder source retaining optional formula,
damage type, and entered settings. It does not join automated damage/recovery episodes.
Completing its details uses the same guarded condition update and episode admission
as other automated sources. Reminders remain visible and removable at the table.
Persistent damage retains complete sources within typed episodes. Select the greater amount
only when comparison is established; otherwise save the GM's choice. Apply active
damage types simultaneously through one HP event, then resolve recovery separately.
Successful recovery clears its episode; individual expiry/removal can reveal a
weaker source. Physical-table recovery uses direct condition removal.
Explicit recovery rolls are independent of initiative progression; durable request
receipts deduplicate retries without imposing a once-per-round allowance. Only
end-turn recovery claims its boundary. Recovery dice and outcomes remain in the
anchored recovery popover; physical-table recovery uses ordinary condition removal.
Outcomes and boundary claims provide replay-safe review.

The compact attack roller remains immediately usable. Optional attack assistance
projects applicable conditions and checks from the shared rules owner. Its flat-check
rolls are reference dice and never assert that target detection, an attack, or an
action has been resolved. Unknown table facts remain reminders instead of a required
questionnaire. No action-attempt API or persistence is retained for an unused
workflow.

Next turn advances atomically before the browser presents one notification for the
turn just completed. Closing by Escape, backdrop or the close icon finishes the
interaction without changing HP, conditions or initiative. Reads and refreshes do
not reopen it. There is no acknowledgment, past-turn queue, selector or pagination.
Durable advance receipts retain the original participant and frozen source context
for retry safety while the current notification is open. Only the latest completed
boundary is actionable; advancing again supersedes it. Receipts remain storage
history, not a reminder inbox. Start-turn Dying, Stunned and waking checks remain
explicit inspector actions, not silently replayed outgoing events.

Observer-relative effects derive directly from active condition sources and their
scope. Hidden/Undetected produce only the affected observer's defense and targeting
context; absence never means Observed. Invisible alone does not establish detection.
Controlled retains source attribution and instructions; conflicting instructions
remain a table decision. There are no saved baseline/adjudication layers, relationship
editing APIs or observer-event automation without a UI workflow.

One condition-system migration follows the published party migration. Unpublished
prototype states do not become historical upgrades or compatibility paths.

## Consequences

Rule changes belong in the shared owner with cross-consumer acceptance cases.
Transport handlers remain thin; frontend forms collect intentional typed inputs.
Saved-state tables do not evaluate PF2e rules. Source adapters do not execute a
general Foundry rules or macro engine. Untracked equipment and behavioral choices
use specific reminders or adjudication, not invented automation.

This extends ADR0048's atomic workflow contract. Old condition arithmetic and
implicit action-intent bypasses must be removed as part of the implementation, rather
than retained as compatibility paths. Full coverage and validation are tracked in
the current condition-system plan; this decision does not itself certify completion.
