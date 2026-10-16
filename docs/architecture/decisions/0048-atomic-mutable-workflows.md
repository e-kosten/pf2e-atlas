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
remaining duration after Continue.

HP operations carry raw input and explicit damage, healing or exact-correction
intent rather than client-computed replacement HP. App-service evaluates the
bounded integer addition/subtraction shortcut against the revision-checked current
participant and reuses the same provenance/replacement policy. Local-state checks
that revision again under the write transaction, so concurrent changes cannot
commit calculations made from stale HP. Exact corrections preserve manual outliers;
derived-maximum reset retains the existing provenance and deficit policy. No new
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
semantics for concurrent edits to list or encounter metadata, an idempotency key
for a response lost after commit, or new gameplay automation. Those require
separate contracts if concrete consumers need them.

## Encounter boundary review and durations

Encounter rows have a monotonic durable revision advanced by metadata, roster,
participant, condition and turn mutations. A side-effect-free turn review reads
one coherent local snapshot and returns that revision with the planned next actor
and reminders. Continue checks the revision under the same immediate transaction
that applies pure app-service timing policy and the next turn. Replaying a review,
including after a complete initiative wrap, conflicts instead of ticking twice.
A receipt contains the canonical encounter key; reference hydration never follows
a committed timing write inside the command.

A condition duration stores one amount and its unit. For rounds the amount is
remaining, and an explicit participant/start-or-end anchor identifies the event
that decrements it. Nonround units are table reminders. Source attribution does
not imply a timing anchor. Legacy rounds retain their unit and remaining amount
with unresolved timing after the atomic mutable-state migration; unrelated edits
can preserve them without guessing an anchor. Missing or removed anchors never
tick. Manual Set turn, reorder, removal, reads and retries are not timing events.
A genuine advance processes the old actor's END and the new actor's START once;
entry without an old actor has only START, and no eligible next actor has neither.

Zero rounds derive pending expiry and retain the condition and its effects.
Review forecasts an upcoming expiry before Continue; Remove now is an independent
saved condition deletion, so Cancel does not undo it. Continue can instead retain
the newly zero duration. Canonical Persistent Damage adds an owner-END table
reminder, with damage/HP and exact condition clearing as explicit separate actions.
The service does not roll dice, apply damage, resolve recovery, delete expired
conditions, or decay severity automatically. Conditions without explicit rule
identity or typed source timing require the user to choose their anchor.

The [round-duration rule](https://2e.aonprd.com/Rules.aspx?ID=2378) distinguishes
creator-START from detrimental-through-target-turn END timing; applicability
cannot be inferred from a condition's display name. The canonical
[Persistent Damage rule](https://2e.aonprd.com/Conditions.aspx?ID=86) supplies its
owner-END reminder and table recovery sequence, independently of duration expiry.
