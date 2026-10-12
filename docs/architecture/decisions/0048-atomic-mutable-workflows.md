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
edits verify their participant owner inside the write transaction.

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
