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

Standard condition identities, value metadata, numerical declarations, grants and
overrides come from the pinned generated condition catalog in `atlas-foundry-model`.
The private source-contract tool also extracts the complete supported trait-to-immunity
helper into guarded declarations. New trait branches using supported syntax regenerate
without trait-specific application code; unsupported control flow or rule fields stop
generation. Reviewed adapter/lifetime dispositions account for source declarations
that need explicit Atlas semantics. Product kind applicability, scope, automation
labels and exceptional lifetimes remain application policy.

One selector/value/predicate adapter serves statistics, strikes, spells, damage,
initiative and effective HP. Generated declarations are immutable source facts,
not executable JavaScript or a second general rule engine. The catalog has its own
pack-byte/declaration/coverage identity, independent of the DTO snapshot contract.
The generated damage-type vocabulary supplies both editor choices and source-fact
keys. Consumer code does not maintain another list or enum-to-key mapping.
Normal execution needs neither Node nor the source checkout.

One checked effective-immunity adapter combines authored entries and generated
trait rules for condition applicability and persistent damage. It preserves authored
exceptions and the engine's authored-entry precedence. Known positive facts survive
partial data; incomplete authored lists and uncertain trait rules remain explicit.
Unknown traits never become an empty trait list. Effect immunity such as mental
immunity does not become blanket immunity to all conditions it could cause.

Independent saved condition sources retain identity, scope, parameters, duration
and origin. Resolve scope before suppression and derive maintained grants without
saving duplicate child rows. The resolver carries known and uncertain causes
together, applies recipient immunity to every maintained child and observer-relative
grant, and retains the parent when only its child is immune. A saved immunity
override admits that source only; it does not waive a different child immunity.
Possible overrides make the affected root and its maintained children uncertain
before downstream predicates are evaluated. The two-stage expansion is bounded
by a catalog contract: an effective override target and its maintained descendants
cannot themselves supply an effective suppressor. A graph test guards this
assumption when the pin changes. Predicate cycles make the varying causes uncertain
without dropping unrelated known causes. Removing one source cannot remove an
independent effect. Persistent consequences, such as Drained HP loss and falling
Prone, use explicit event plans. Correcting an entry differs from applying an effect now.
Behavioral subjects, such as a fascination or threat, are distinct from the source
that applied the condition. Missing subjects require context only when an action
depends on them; the causal source is not an implicit substitute.

Source-owned conditions are distinct, read-only causes. Checked owned conditions,
maintained in-memory grants and unlimited owned effects feed the same cause
resolver as temporary saved conditions. Statistics, HP limits, senses, action
restrictions, regain forecasts, targeting and lifecycle calculations consume those
resolved causes. `condition_causes` is the shared effective-condition accessor;
`known_presence`, `known_value` and bounded comparisons preserve uncertainty.
Saved-row lookups select edits or captured history rather than effective arithmetic.
They retain the
participant, root, checked owner, rule ordinal and source fingerprint; they never
receive synthetic saved condition IDs. Constant badge overrides preserve supplied
magnitudes. Missing values and uncertain activation remain explicit. Conditions
owned by a creature, such as Sickened Doprillu's Sickened, remain source-linked
and read-only; a separate temporary condition remains independently editable. A higher
temporary Slowed value does not erase the inherent source; removing it exposes
the inherited value again. Turn-start, explicit regain and Spend prepare reference
facts before the guarded write and check the reference revision. Spend honors known
source restrictions without reallocating stock; Restore and Correct remain manual
corrections. Unavailable source facts retain manual tracking when no known prohibition
applies. A rules-owned RegainPlan evaluates allowance, Stunned loss, allocation,
Quickened restrictions and typed missing facts together. Presentation adapts that
plan; allocation never consumes display notes or DTOs. Regain and validated explicit
Stunned recovery use one pure consumption reducer.
Matching turn receipts recover before reference loading or that guard. Recorded
stock survives later source changes; subsequent forecasts use the current source.
Each successful allocation also saves typed calculation facts with the turn stock,
before consuming Stunned. While that turn is open, the Actions explanation uses
those recorded facts, including Slowed/Stunned overlap. Later automatic losses are
recorded separately from the initial allotment; manual spending/restoring does not
rewrite it. When the turn ends, the explanation returns to the live regain forecast.
Older turns without allocation facts retain usable stock and explicitly unavailable
historical detail; reads never reconstruct that history from current conditions.
Before a creature's first turn, reaction tracking starts ungranted. An explicit
GM grant saves reaction stock without allocating actions or starting a turn.
That stock supports the usual spend/restore controls and refreshes normally when
the first turn begins. Persisted pre-turn stock remains distinct from an allocated
turn, including when regaining actions is pending.

Grant immunity uses the shared checked immunity policy. Bounded condition
predicates and supported authored RollOption switches/finite condition choices
select maintained causes. Participant overrides persist separately from derived
conditions with stable owner/rule slots and versioned declaration identities. The
identity hashes supported option, choice and linked-grant semantics, or frequency
capacity/interval; display labels, whole-corpus fingerprints and authored initial
values are excluded. Unrelated reference edits preserve saved state. Changed or
removed declarations, moved slots and legacy entries without that identity use
the current authored defaults; Atlas does not silently rebind them or create a
confirmation workflow. Navigation fingerprints still guard commands against stale
source views. All ability-state writers, including turn and overnight refresh,
store the current declaration identity. The pinned finite-choice
preparer supplies its first authored choice when no selection is stored; toggles
use the authored value or pinned default. Equipment requirements, context-dependent
expressions and unsupported effect lifetimes remain authored rules. Prose-only
restrictions, including Zombie Slow's reaction prohibition, are
not inferred from condition identity, creature names or traits.

Structured ability frequency exposes authored capacity/interval and independently
saved encounter expenditure. Use and Restore are explicit, independent of dice and
action/reaction tracking. Initiative advancement refreshes per-turn frequencies
for all participants and per-round frequencies at their owner's turn start, matching
the pinned Foundry recharge policy. Longer intervals restore explicitly because
Atlas has no game clock. Reads and unrelated source refreshes do not reset bound expenditure.
Turn advancement prepares incoming source conditions and frequency refreshes in
one reference snapshot before the guarded write; it never re-resolves a mutable
encounter slug between preparations. Ordinary resolved damage that needs no
source lifecycle facts, and removal of an unmodeled custom condition, remain
state-local operations. Healing still resolves source conditions to preserve
effective maximum HP. Events that can reach zero HP also prepare source mortality
facts: source-owned and saved Petrified both destroy the statue at zero, including
when dying tracking is disabled or persistent damage uses table totals. Table totals
remain after protections and are never reduced by IWR a second time. Unknown
Petrified applicability blocks that mortality decision rather than inventing an
ordinary knockout. Saved-row recovery selectors remain separate from effective
condition truth; source-owned conditions are read-only.
Explicit saved-condition and source-option changes share one before/after
consequence planner. It resolves supported predicates after generated condition
changes before calculating Drained loss and mortality; switching Drained off does
not heal HP. Own-turn Stunned changes use effective sources with the recorded
remaining allowance and the same saved-source consumption reducer. Immutable
causes are never reduced through fabricated saved rows. Healing or recovery awards
Wounded only when effective Dying actually ends, using the resulting condition facts.
Unknown facts block only calculations whose result depends on them, before RNG or
writes. A table degree still needs condition facts to calculate its consequences.
Pending decisions with unproven freshness remain available with an explanation;
source-independent correction remains possible. Exact maximum HP and overnight
admission cannot treat unavailable source data as known empty. Dominated or capped
uncertain values do not hide a result that is already established.
New source-derived participants start at their effective capacity while retaining
baseline maximum HP. Initialization completes inside the creation transaction and
never rewrites an existing participant. Source-derived capacity is never charged
again by reading or refreshing a record.
Breath recharge prose and inline roll labels do not establish structured cooldown
semantics; no cooldown or reset trigger is inferred from names or descriptions.

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

A direct attack roll retains backend numerical adjustments and source/request
guards. Optional Target & checks assistance is read-only, stays beside the original
roll and cannot reroll or rewrite it. The spell popover reads condition reminders
and records a direct guarded use through the existing resource mutation owner.
Recording use asserts expenditure, not targeting success, completed checks or
permission to act. Reference flat checks are dice for the GM, not a saved action
state machine. Retch's fixed traits feed the shared evaluator. Unresolved
restrictions remain hints; recovery does not collect action judgments. Integrated
Retch rolls require known source DC, current Fortitude and resolved restrictions.
A table outcome records the GM's resolution without replaying its prerequisites.
The action evaluator covers the live Strike, Cast and Retch workflows. Actions
the product cannot initiate do not have query branches.
There is no general action-query API, durable attempt history,
frozen action context or alternative cost-commit path.

HP events distinguish resolved damage, resolved healing, direct loss and exact
correction. Already mitigated table amounts never receive defenses a second time.
Saved maximum HP remains baseline capacity; condition-adjusted capacity is derived.
One attributed maximum-HP projection feeds source-backed statistics, manual-PC HP,
the inspector summary/meter and healing limits. Missing condition inputs leave the
effective cap unknown; a known baseline is never treated as an unlimited healing
cap. Direct current/temp edits are exact corrections and do not replay damage or
healing consequences. Drained arrival previews stay anchored to the changed row;
missing level is supplied once there and remains a saved participant fact.
Track Dying and mortality are distinct from Defeated/Disabled participation markers.
The Dead/Defeated status entry can start dying tracking at0HP through the same
knockout planner, preserving HP and atomically applying the chosen outcome. An
explicit Markalive correction preserves the tracking preference and retires stale
HP/initiative decisions. Source applicability and the reference revision guard
still apply. Backend-projected knockout choices preserve known damage facts;
initiative placement remains editable and is committed with the knockout outcome.
Pending HP decisions are applicable only while their recorded HP/dying state remains
current. Reads hide obsolete decisions without changing saved state. Encounter
events and direct participant edits retire them within the guarded write that
invalidates them, so returning to 0 HP cannot revive an older knockout. Storage
only deletes the workflow-selected keys; app-service owns applicability. Closing
the knockout popover leaves a current decision available; there is no separate
Skip action.
The UI suggests the active turn, rather than identifying it with the damage source;
no active turn defaults to keeping position. Initiative-only entries use the
concrete Moveinitiative action rather than a generic HP decision form.
Explicit level overrides and death thresholds are saved participant facts, not
values inferred from notes or transient form state.

Incomplete persistent damage is a typed reminder source retaining optional formula,
damage type, and entered settings. It does not join automated damage/recovery episodes.
Completing its details uses the same guarded condition update and episode admission
as other automated sources. Reminders remain visible and removable at the table.
Persistent damage retains complete sources within typed episodes. Select the greater amount
only when comparison is established. Otherwise the default precedence uses higher
additive-dice averages without claiming a rules comparison. Add/Edit exposes the
complete source order whenever any pair is ambiguous, including fallback sources
below a uniquely strongest source. Apply saves that order and the condition in
one event. Reads ignore absent members while preserving every survivor's relative
order; membership changes do not erase lower preferences. New sources enter their
default positions; formula/recovery edits reconsider only the changed source,
while metadata-only edits retain the whole order. Damage and recovery consume
precedence without replacing it. The rules owner returns nonempty groups with
one active source and the complete source order. Projection, damage and recovery
consume that same selection. Damage and recovery requests identify episodes;
the guarded backend snapshot supplies their active source. Source comparison and ordering
belong in Add/Edit, with no second source picker in damage or recovery. Storage resolves typed addition references after
allocating their IDs in the event transaction; there is no intermediate write. Apply active
damage types simultaneously through one HP event before condition removal. The
turn reminder can prepare each damage roll and optional recovery check first.
Unrolled expression shapes are projected from the existing validated Rust dice AST;
they consume no randomness and contain no result or kept/discarded claim. Arithmetic structure is preserved; the
original formula retains keep-filter context. The browser renders
these shapes without parsing formulas or constructing fictional roll outcomes.
Receipt-only preparation preserves HP, conditions and encounter revision, with
separate per-type preparation claims. Manual amounts and Clear choices remain
frontend drafts. The final guarded damage command consumes prepared receipt IDs,
checks exact boundary/source/formula/generation identity, recalculates current
protections from retained raw dice, applies shared Hardness and HP consequences
once, and then commits queued recovery/removal in the same event. A changed HP
value does not require rerolling damage. An altered formula or selected source
cannot reuse unrelated dice; table resolution remains available. Actual recovery
checks retain their rolled DC and once-per-boundary claim; they are not simulated
as table outcomes or silently omitted at final submission. Ordinary recovery after
applied damage remains an independent guarded action.
Successful recovery clears its episode; individual expiry/removal can reveal a
weaker source. Physical-table recovery has a per-type Clear intent beside the recovery roller.
Before damage application it queues removal, visibly retaining this turn's damage;
the final command records that table outcome with the batch's recovery claims.
If an actual check was already prepared, Clear still removes the effect while
retaining that check's DC, dice and success in canonical review. Removal does not
rewrite a failed check as a successful or unrolled result.
After damage application, or outside the prepared batch, direct Clear ends the
recorded episode's matching current sources without a DC, source-selection form,
simulated roll, HP change or turn-recovery claim. Both forms preserve other types
and sources added after the outgoing boundary. The review
and damage command share an executable-source projection intersecting the recorded
sources with current episode membership, so ended sources cannot be charged later.
Review responses pair those groups with their encounter revision so clearing a
source cannot invalidate the remaining batch while the parent turn view refreshes.
Explicit recovery rolls are independent of initiative progression; durable request
receipts deduplicate retries without imposing a once-per-round allowance. Only
end-turn recovery claims its boundary. Recovery dice and outcomes remain in the
anchored recovery popover; individual source rows retain ordinary removal.
Outcomes and boundary claims provide replay-safe review.

The compact attack roller remains immediately usable. Optional attack assistance
projects applicable conditions and checks from the shared rules owner. Its flat-check
rolls are reference dice and never assert that target detection, an attack, or an
action has been resolved. Unknown table facts remain reminders instead of a required
questionnaire. No action-attempt API or persistence is retained for an unused
workflow. Casting reminders read against the encounter revision because condition-only
changes do not advance the participant resource/settings revision. Retained advice
is valid only while the spell, source generation, participant conditions, source
option states and status remain unchanged. Source options retain checked owner,
rule ordinal and fingerprint identity. Switching a condition-granting option retires
previous advice immediately, including during failed refreshes; spending an ability
use or a spell resource preserves still-valid advice and the anchored spell view.

Next turn advances atomically before the browser presents one notification for the
turn just completed. Closing by Escape, backdrop or the close icon finishes the
interaction without changing HP, conditions or initiative. The browser pins the outgoing
participant inspector while reminders are open, retaining live canonical updates
for that participant. Dismissal follows the saved current turn; a quiet boundary
follows immediately. This display choice does not defer the atomic turn advance.
Reads and refreshes do
not reopen it. There is no acknowledgment, past-turn queue, selector or pagination.
Durable advance receipts retain the original participant and frozen source context
for retry safety while the current notification is open. Only the latest completed
boundary is actionable; advancing again supersedes it. Completed condition reminders
remain as typed Resolved results derived from current source membership or recorded
boundary claims. Removed participants are distinguished from removed effects.
Persistent type cards own their existing damage/recovery results; ordinary duration,
timing and end-turn recovery rows follow the same retained-result policy. Reads
neither recreate actions nor infer successful writes from frontend drafts. Receipts remain storage
history, not a reminder inbox. Incoming deterministic regain changes are retained
as concise results beneath Turn controls in the runtime lane; ordinary starts remain quiet.
Dying recovery remains an explicit inspector action and defers regain. Supported
numeric Stunned is consumed at actual regain in the atomic advance; a new numeric
Stunned applied during an open current turn invokes that same pure reducer within
the condition application event against recorded remaining actions. The before/
after preview and commit use one plan. A source consumed before insertion has no
saved row ID; the application receipt and retained turn result preserve the event
without persisting the preview subject's temporary ID. Surviving sources retain
their notes, scope, origin and actual saved identity. A separate consumed-application
marker retains replay protection when there is no surviving source row. Neither
path is replayed by reading an outgoing event.
Waking from
noise is guidance alongside the existing Perception roller, not a DC-entry form.

Observer-relative effects derive directly from resolved saved and authored condition
sources and their scope. Possible causes remain uncertain through recipient immunity.
Hidden/Undetected produce only the affected observer's defense and targeting
context; absence never means Observed. Invisible alone does not establish detection.
Controlled retains source attribution and instructions; conflicting instructions
remain a table decision. There are no saved baseline/adjudication layers, relationship
editing APIs or observer-event automation without a UI workflow.

One condition-system migration follows the published party and trait migrations. Unpublished
prototype states do not become historical upgrades or compatibility paths.

Condition eligibility reads typed immunity coverage and hazard health facts directly
from checked source views. It does not hydrate record presentation or correlate
presentation entries by position. Subject facts use the caller's participant snapshot;
source and encounter revisions guard the eventual write.

Recovery offers are action-specific data, not a nullable field bag plus a parallel
required-input schema. Rust associates offers with saved condition source IDs;
one browser recovery owner presents hints and meaningful actions in condition
rows. Pending HP decisions stay beside HP controls.
The compact manual Turn controls and their retained results share the runtime
lane with conditions beside HP and defenses. In a single column they follow
defenses and precede statistics. Condition-specific recovery stays in its condition
row rather than becoming an unrelated action/disclosure collection there.
Recovery hints/results remain anchored to the requested action, including while
successful resolution removes the associated condition. Listing and check
submission share the offered source facts. Known checks use the shared dice
surface; table outcomes use direct controls without raw-result entry. The recovery
API accepts Roll or a table degree; the unused raw total/natural input is retired.
Historical check outcomes retain their stored table totals. Missing DCs,
modifiers, elapsed time and recovery reasons never become required recovery fields.
Wounded, waking and overnight recovery use hints and existing row controls rather
than separate validation commands. Broken uses its derived/source threshold and
normal condition editing; missing thresholds do not queue a recovery form.
A request-local prepared lifecycle plan validates source decisions before RNG and
resolves the actual check outcome once; it is never saved as an attempt. The concrete
encounter event executor owns receipt recovery, reference/participant checks and
entry to local-state's guarded callback. Storage remains the sole atomic write owner.

Recovery prepares checked source facts for Dying consequences and Stunned action
loss, including when the remaining action stock is already recorded. An integrated
Retch roll also needs current Fortitude. Retch table degrees and saved-only
Frightened recovery retain their independent input policy. When required source
facts are unavailable, unrelated offers and manual corrections remain usable;
Atlas does not invent a condition value or an action allowance. Outcome validation
retains the specific rule message and authored source label through the atomic
boundary; failed outcomes save no receipt or boundary claim. A bounded known loss
can be recorded without claiming an exact Stunned magnitude; its historical
adjustment and remaining-condition preview preserve that distinction.

Condition editing has one control-to-command adapter and one session owner for its
request draft, query-backed preview and submission. Semantic preview effects identify
their participant independently of storage change ordering. Persistent edit inputs
retain optional formula/type but carry no resolved episode identity. Rust admits
reminders versus complete sources and selects or preserves the canonical episode.
Stored states remain distinct. HP damage/healing uses the product HP action command;
resolved damage, healing and direct loss remain internal policy inputs.

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

A sense-dependent Perception penalty does not erase an otherwise known statistic.
The rules projection retains the known value and attaches its conditional explanation
to that statistic; authored sense precision/range and condition availability remain
visible alongside it. No target distance is invented for an untargeted overview.

The action-regain explanation shows each condition once with its actual contribution.
Fully covered Slowed is suppressed; partial coverage includes a numeric explanation
from the rules owner. Quickened's extra-action status assumes losing that action
first when actions are lost. This is display policy, not saved action allocation or
a spending ledger. Recovery remains an explicit condition-row operation.

Persistent damage review projects the same per-type protections used by the damage
command. Immunity, weakness and resistance attribution stays beside the damage
entry/result; applied roll receipts retain those facts rather than inferring a
past result from today's reference data. Table totals bypass defense automation
and carry no implied protection calculation. Effect admission immunities (including
sleep, disease and fear) do not block an already-recorded persistent damage packet;
unknown damage-relevant selectors and exceptions still require a table result.
Non-random end-turn Frightened reduction is a direct reminder action, without a
roller/popover step. Guarded revisions and boundary claims still protect retries.

### Attitude and behavioral reminders

Attitudes can be recorded without confirming their supernatural origin. A PC attitude
form shows a concise rules reminder; it does not require an extra admission fact or
an immunity override solely because the participant is a PC. Condition immunity
handling remains separate. Attitude parameters carry no supernatural flag.

Behavioral condition row hints and action-specific casting reminders are projected
by the shared rule owner. Persistent damage group projections may include an immunity
note established by the same damage-protection evaluation used for resolution;
the frontend renders that fact without reinterpreting trait or immunity rules.
