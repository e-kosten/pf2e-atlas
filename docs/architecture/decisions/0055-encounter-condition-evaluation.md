# ADR 0055: Shared Encounter Condition Evaluation

Status: accepted; implementation and validation in progress.
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

`atlas-local-state` owns atomic event writes, receipts, pending consequences and
action attempts. A write recovers an existing receipt before checking revisions,
then validates the current snapshot before rolling or changing state. The same
request cannot repeat HP loss, resource expenditure, checks or turn processing.
Accepted boundaries also have unique participant/round/scope claims, so changing
the request ID cannot repeat the boundary. Reads and manual turn selection are
not lifecycle events.

Optional evaluated spell and strike attempts share an action-attempt lifecycle.
Ordinary table play does not require that workflow: a direct attack roll retains
backend numerical adjustments and source/receipt guards, with relevant advisory
context rather than mandatory target adjudication. Optional attack assistance stays alongside
the validated original roll receipt and retains its dice through target selection and reference checks;
it must not silently reroll or rewrite historical evidence. The spell popover shows relevant
reminders and records a direct, guarded use. Recording use asserts resource expenditure,
not successful targeting, completed checks or permission to act. It retains exact source
and resource validation and retry safety. Frozen typed context for an evaluated attempt records
the source generation, participant facts, judgments, costs and ordered checks.
Completed checks and terminal results survive reopening. Failure or abandonment
does not refund a committed spell cost. Atlas does not maintain a current-action
counter: action spending remains at the table. Table-resolved use, resource correction
and evaluated attempts are distinct explicit intents; none fabricates evidence for
another.
Secret targeting checks retain their evidence for the GM. A separate shareable
summary omits the secret cause and result details. This is a GM-local presentation
boundary, not a player access-control system. Guessed-location correctness is an
explicit frozen table fact, never inferred from encounter roster membership.

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
weaker source. A table-success outcome records success without inventing a die result.
Explicit recovery rolls are independent of initiative progression; durable request
receipts deduplicate retries without imposing a once-per-round allowance. Only
end-turn recovery claims its boundary. Recovery dice and outcomes remain in the
anchored recovery popover; physical-table recovery uses ordinary condition removal.
Outcomes and boundary claims provide replay-safe review.

The compact attack roller remains immediately usable. Optional attack assistance
projects applicable conditions and checks from the shared rules owner. Its flat-check
rolls are reference dice and never assert that target detection, an attack, or an
action has been resolved. Unknown table facts remain reminders instead of a required
questionnaire. The existing durable action-attempt API remains available to evaluated
action workflows; the reference attack UI does not enter that state machine.

Next turn advances atomically before the browser presents one notification for the
turn just completed. Closing by Escape, backdrop or the close icon finishes the
interaction without changing HP, conditions or initiative. Reads and refreshes do
not reopen it. There is no acknowledgment, past-turn queue, selector or pagination.
Durable advance receipts retain the original participant and frozen source context
for retry safety while the current notification is open. Only the latest completed
boundary is actionable; advancing again supersedes it. Receipts remain storage
history, not a reminder inbox. Start-turn Dying, Stunned and waking checks remain
explicit inspector actions, not silently replayed outgoing events.

Directed relationships separate saved baselines from active condition causes.
Unknown, conflicting and invalidated knowledge are explicit; absent detection never
means Observed. Observer-specific transitions preserve other observers' sources.
A new Invisible source invalidates an older detection assessment. Source sets are
retained when a GM adjudicates simultaneous causes. Baseline and adjudication are
separate durable layers: adjudicating conflicting causes preserves both sources and
the prior baseline. The decision is valid only for its exact typed source snapshot;
ending all causes reveals the baseline again. Formal Request eligibility
uses resolved attitude without inventing success of its Diplomacy check.

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
