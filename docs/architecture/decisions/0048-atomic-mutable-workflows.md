# ADR 0048: Atomic Mutable Workflows

Status: accepted.
Date: 2026-10-12

## Context

Saved lists and encounters persist user intent separately from the rebuildable
reference artifact. A product action can change several local rows. Serializing
individual SQLite statements does not make that action atomic, and serialized
writes do not protect against complete replacements built from stale reads.
Reference projection can also fail after a local write has already committed.
Reporting that read failure as a failed mutation invites duplicate actions.

## Decision

`atlas-local-state` owns transaction boundaries for durable product operations.
Initialization and migrations, including version metadata, commit together.
List metadata and tags, quantity additions, and participant removal with turn
replacement commit as complete actions. Aggregate reads use one read snapshot;
returned local rows are read before the write transaction commits.

Participant rows have monotonic revisions. Complete participant replacements
require the revision the caller read. App-service checks it before source-based
derivation, and local-state checks it again under the write transaction. Stale
requests receive a typed conflict without changing state. Reordering advances
revisions of affected participants. Conditions use non-reused IDs, and condition
edits verify their participant owner inside the write transaction. Full condition
replacements also require the encounter revision the editor read, checked before
validation and again inside the write; a stale notes draft cannot restore an older
remaining duration after turn advancement.

HP operations carry raw input and explicit damage, healing or exact-correction
intent rather than client-computed replacement HP. App-service evaluates the
bounded integer addition/subtraction shortcut against the revision-checked current
participant and reuses the same provenance/replacement policy. Local-state checks
that revision again under the write transaction, so concurrent changes cannot
commit calculations made from stale HP. A single initial plus/minus sign extends
the bounded integer shortcut. Signed Heal intent handles Enter; negative Damage
or Heal amounts reverse the operation before running the actual damage/healing
policy, including temporary HP, floor/cap and defeated behavior. Exact corrections
keep their zero floor and preserve manual values above the maximum;
derived-maximum reset retains the existing provenance and deficit policy. The
three-field HP editor submits one SetValues operation. Omitted current/maximum/
temporary fields preserve their values and edit intent; supplied current/maximum
wrappers carry raw formula input or explicit null for unknown. All supplied
inputs are interpreted before one guarded participant replacement, including
same-value explicit maximum intent. No sequential partial saves are used. No new
storage schema or expression/rules engine is introduced.

App-service owns turn eligibility and selection policy. Local-state supplies
the locked encounter snapshot to a narrow pure callback and applies the chosen
turn in the same transaction. Callbacks do no source retrieval or artifact I/O.
There is no generic repository or unit-of-work facade.

Existing-encounter mutations return a durable receipt containing the encounter
key. They do not hydrate reference presentation after commit. Create and delete
retain their identity receipts. Saved-list import reuses already resolved
reference summaries when composing its receipt. Additions retain the canonical
encounter key resolved before source work, rather than resolving a mutable slug
again at write time. Reads independently hydrate current presentation; a failed
read cannot turn a committed write into a failed write.

Encounter detail and edit routes share one route-local mutation owner. It keeps
controls pending through the write and canonical-key refresh, updates the route
alias cache from that read, and prevents duplicate submission. It does not
optimistically calculate participant state. A committed write followed by a
failed read shows a Saved warning and disables further edits until a read-only
Retry succeeds. A typed participant conflict refreshes current state while
retaining the failed draft; retrying the edit requires another user action.
Other failures remain attached to the initiating action.

Editors await Save before navigating. Destructive confirmations await deletion
and remain open on failure. The encounter index owns its separate asynchronous
deletion lifecycle rather than instantiating a detail-route mutation owner.
Same-entity background refreshes preserve touched metadata drafts; changing
entity or editing session resets them. Route owners prevent old completions
from navigating a newer editor. Initial read failures have local Retry controls.

## Consequences

Existing state migrates in place without resetting HP, provenance, conditions,
lists, or ordering. An unsuccessful action leaves its prior state intact.
Independent clients can detect stale participant replacements. Consumers refresh
presentation separately after receiving a commit receipt. No generated-artifact
schema or gameplay rule changes are implied.

Revision checks cover participant replacements. This does not introduce merge
semantics for concurrent edits to list or encounter metadata. Roll requests use
the narrower idempotency contract below; other mutations do not gain it implicitly.

## Dice and initiative receipts

`atlas-dice` validates and evaluates bounded formulas through `diceng`, retaining
an arithmetic result tree with actual die sides, faces and kept/discarded state.
It has no Foundry, SQLite or application workflow dependency. Browser drawing and
motion consume results and cannot determine rolls. The supported initial grammar
is ordinary dice, signed integer arithmetic, parentheses and keep-high/keep-low;
unsupported source notation must not be silently approximated.

App-service resolves initiative statistics through the existing encounter overlay
policy and accepts explicit manual modifiers. Unlisted NPC skills use the source
system's ability baseline before condition/variant adjustments. Unknown inputs
remain unavailable for automatic rolling. Each participant rolls independently.

Initiative options return the artifact generation used to hydrate their source
modifiers. Source-based rolls must submit that generation, compared against the
same hydration snapshot that resolves the roll. Saved-receipt recovery precedes
this guard, so publication of a new artifact cannot reroll a committed request.
Explicit manual-modifier requests remain source-free and need no generation.

The condition-system schema stores encounter roll receipts. A canonical encounter key
and request ID identify an operation; the original intent is saved alongside the
resolved inputs and outcomes. Matching retries recover the receipt before source
hydration and revision checks. Reusing an ID with different intent conflicts.
The immediate write transaction repeats recovery, validates the complete batch,
evaluates and saves each initiative using existing tie-order policy, then stores
the receipt. Any failure rolls back the whole batch. Source/artifact I/O remains
outside that transaction. Current turn, HP and spell resources are unchanged.

Persistent damage/recovery preparation uses the same guarded receipt storage with
an explicit receipt-only write. Its callback can return outcomes only, so it
cannot change HP, conditions, pending consequences or the encounter revision.
Independent preparation claims deduplicate per-type rolls, while the ordinary
damage/recovery claims remain unconsumed until final application. Read-only draft
previews do not write receipts or roll dice. ADR0055 owns source binding and the
single final HP/recovery event.
Final application reads and validates preparation claims in its workspace write
snapshot after receipt recovery. Encounter revision alone cannot detect a new
receipt-only preparation; no earlier read can authorize omitting that result.

Receipts are retained for the encounter lifetime and cascade on encounter deletion;
there is no automatic time-based pruning and no recovery guarantee after deletion.
Consumers refresh independently after a successful receipt. Retrying a lost
response uses the same request ID and intent, not a fresh roll request.

Strike rolls resolve the selected embedded activity through the same encounter
projection used for display. Variant and condition adjustments belong in that
projection; the roll workflow consumes its effective attack modifiers and damage
formulas. It must not disable otherwise supported rolls to avoid implementing
those adjustments. Attack number is an explicit choice, not inferred from prior
clicks. Damage components retain their type, category and original dice outcomes.
Damage-domain totals are derived in app-service, separately from generic dice
arithmetic and from any eventual target damage application.
The pinned PF2E 6.12.4 source supplies trait-based attack ability and strength
damage applicability. NPC adjustment affects the first numeric damage component;
applicable status penalties retain their normal stacking policy. Same-type damage
components form an instance floored at zero, with the overall minimum represented
explicitly. Persistent instances remain separate from immediate damage. Empty
source damage formulas do not acquire damage through adjustment arithmetic.

Strike receipts use the same encounter/request identity with an action
discriminator. Their transaction checks encounter and participant revisions and
stores the outcome without changing initiative, HP, turn or spell resources.
Recovery precedes source hydration, so a lost response cannot reroll merely
because the source or participant subsequently changed.

Reference table rolls are stateless app-service reads. A checked artifact
generation and selected table supply the formula and inclusive result ranges;
all matching rows retain their prepared content and checked references. Gaps and
invalid ranges are explicit. Rolling does not consume table entries or recursively
roll linked tables. A subsequent reference roll is a new roll, not recovery of a
durable encounter operation.

## Encounter boundary review and durations

Encounter rows have a monotonic durable revision advanced by metadata, roster,
participant, condition and turn mutations. Next turn checks the opening revision and
request identity in the same immediate transaction that selects the eligible next
actor, applies timing policy and saves the new turn. Participants without initiative
are skipped. A matching retry recovers its receipt; a distinct stale request conflicts.
The transaction saves outgoing context with original participant, round and condition
source/episode identities. Reference hydration never follows a committed timing write
inside the command.

The browser advances first, then presents one optional notification for the turn
just completed. Every close path finishes the notification without changing HP,
conditions or initiative. There is no pending-reminder queue or acknowledgment.
Only the latest completed boundary supports guarded actions while its notification
is open; advancing again supersedes it. Durable receipts retain retry identity and
frozen source context without exposing a past-turn workflow. ADR0055 owns the shared
condition rules used by these actions.
A condition duration stores one amount and its unit. For rounds the amount is
remaining, and an explicit participant/start-or-end anchor identifies the event
that decrements it. Nonround units are table reminders. Source attribution does
not imply a timing anchor. Legacy rounds retain their unit and remaining amount
with unresolved timing after the atomic mutable-state migration; unrelated edits
can preserve them without guessing an anchor. Missing or removed anchors never
tick. Manual Set turn, reorder, removal, reads and retries are not timing events.
A genuine advance processes the old actor's END and the new actor's START once;
entry without an old actor has only START, and no eligible next actor has neither.

Zero rounds derive pending expiry and retain the condition and its effects. Outgoing
review can show those conditions after advancement; removal is an explicit independent
command. Persistent Damage adds an owner-END reminder with frozen due source context;
damage/HP and exact episode recovery are explicit separate actions.
Advancing the turn does not itself roll damage, resolve checks or delete expired
conditions. Explicit lifecycle commands prepare source facts and use pure rules to
plan HP and condition consequences. Local-state commits the complete prepared
changes, typed outcomes, pending decisions and receipt atomically. Boundary claims
prevent a second accepted end-turn batch or recovery under a different request ID.
Matching retries recover the saved receipt before source/revision checks and RNG.
Independent condition sources retain parameters and application provenance.
Application identity remains claimed after removal or consumption before source
insertion. The latter uses a typed consumed-application write with no source row
ID. Local-state claims both forms through the same atomic identity guard, so a new
request ID cannot repeat one application's consequences.
Aggregate mortality counters normalize their active contributors on actual counter
changes. Unconscious entry can create lasting Prone; removing its source does not
undo that consequence. Correction intents do not replay application consequences.
Unknown facts remain pending or require input before a dependent check; they never
become numerical zero. Conditions without explicit rule identity or typed source
timing require the user to choose their anchor.

The [round-duration rule](https://2e.aonprd.com/Rules.aspx?ID=2378) distinguishes
creator-START from detrimental-through-target-turn END timing; applicability
cannot be inferred from a condition's display name. The canonical
[Persistent Damage rule](https://2e.aonprd.com/Conditions.aspx?ID=86) supplies its
owner-END reminder and table recovery sequence, independently of duration expiry.

### Encounter spell resources

The app-service derives casting eligibility and resource pools from each
participant's checked Actor source. The local-state owner stores capacities,
remaining counts, and one undo value per resource under that participant.
Schema 10 adds `encounter_spell_resources`; schema 11 adds participant-owned
`encounter_expended_spells` markers. Artifact storage is unchanged.
Prepared spells have a pool per entry, spell identity and rank. Spontaneous
spells share rank slots within an entry, innate spells use their explicit use
counts, and NPC focus spells share the typed focus pool. Cantrips consume no
resource. Innate spells with absent use counts expose a manual Expended/Restore
marker instead of an inferred capacity. Markers are per participant and stable
spell identity, independent of preview rank; they are reminders, not casting
prohibitions. Rejected or null capacities remain untracked. Source spell
identities must be stable and unique before they can own persisted resources.

New participants start at their full allotment, independently of exported
remaining counts. Casting, Undo, count correction and entry restoration replace
only resource rows in an immediate transaction guarded by participant revision;
they also advance the encounter revision. HP, conditions and settings are not
replaced. A later numeric spell-resource mutation invalidates the previous Undo;
independent expenditure markers do not alter numeric resources or their Undo. Reads,
rank previews and encounter completion never consume or restore resources.
Preexisting participants acquire their source-derived pools on their first
resource mutation. Stored capacities remain authoritative thereafter.

The browser renders the service catalog and sends typed actions through its
existing route-owned mutation/recovery flow. Shared reference presentation
accepts action slots without owning encounter state. Casting does not execute
Foundry rules, target effects, attack rolls or turn advancement.

The resource catalog also identifies display scope: individual spell, shared rank
slots, or shared entry resources. Browser rosters use this scope for count
placement and dim exhausted links without preventing reference reading. Preview
actions use the same resource state. Bounded counts are normalized before numeric
comparisons because JSON numbers and generated TypeScript bigint annotations do
not have the same runtime representation.

Resource management edits one entry as a local draft. Restore all changes that
draft; Cancel and dismissal discard it. Done submits all entry counts in one
revision-checked request, validated in full before the resource transaction.
The editor retains its opening participant revision and preserves its draft on
conflict, rather than rebasing edits onto a newer snapshot without consent.

Shared rank and focus counts also expose a compact immediate editor. Each change
uses the same revision-checked entry replacement, retaining other pool values.
Spell action conflicts refresh without replaying writes and show brief action
feedback; an expenditure marker already in the requested state needs no error.

## Encounter participation and action assistance

Defeated is an encounter participation marker, not a diagnosis of death or
unconsciousness. Damage reaching zero marks creatures defeated and hazards disabled;
healing does not reactivate either. Hazards use Active/Disabled instead of
Active/Defeated, with HP tracked separately. Existing defeated hazards migrate to
disabled while preserving HP and other saved state.
Both inactive states exclude automatic turn selection; reads, edits and explicit
resource corrections remain available. Next turn continues from the current
participant's position even if that participant has become inactive. No fallback
selects defeated PCs when everyone is inactive. Reactivation never heals or
executes authored Reset instructions.

App-service owns the shared condition and action policy described in ADR0055.
A genuine Next turn allocates incoming creature/PC resources and consumes supported
numeric Stunned in the same transaction as outgoing context, duration updates and
turn selection. The recorded initial allowance remains after Stunned decreases or
clears; live condition forecasts never refill that turn. Unknown inputs and pending
Dying recovery defer allocation. Hazards retain their authored routine rather than
receiving generic creature resources. Manual Set turn remains position-only.

Explicit manual spend, restore and exact correction use encounter revision, actual
turn identity and durable request receipts. Attack rolls, spell-use recording and
other mechanics never spend actions or reactions implicitly. Actions expire at the
end of their turn. Reaction stock survives other actors' turns and is replaced only
at its owner's actual regain; permission restrictions do not change spent/unspent
stock. Action edits require that actor's open current turn. Reaction bookkeeping
remains available off turn. Existing encounters begin with unknown allocations;
migration must not invent a past regain or expenditure.

Quickened adds at most one restricted extra action before losses. The existing
extra-first default for Stunned/Slowed is retained; restriction facts remain attached
to that action. Mid-turn Quickened/Slowed change the next-regain forecast. Applying numeric
Stunned during an open current turn uses recorded remaining
actions and atomically reduces both that pool and condition sources in the same
condition event. The GM finishes the current activity before submitting its
condition application. Preview shows both the action loss and remaining Stunned;
matching retries never repeat either change. Correction and off-turn application
do not consume actions. Explicit Stunned recovery remains available for deliberate
resolution of recorded sources. It never reruns a three-action regain. Missing
Stunned value with a duration
is duration Stunned; missing both is unresolved. Pending-expiry review and explicit
removal govern duration Stunned. Reads never consume conditions or resources.
The timing and permission distinction follows [Start Your Turn](https://2e.aonprd.com/Rules.aspx?ID=2428)
and [Gaining and Losing Actions](https://2e.aonprd.com/Rules.aspx?ID=2455).

Evaluated casting uses the backend action judgments and relevant exception facts.
The ordinary spell popover records table-resolved use with source, resource and revision
guards, showing condition reminders without requiring an evaluated action workflow.
Resource editing and undo are distinct bookkeeping actions. Recording expenditure does
not establish spell success, validate permission or execute effects.
