# ADR 0059: Encounter creation baselines and recovery

Status: accepted
Date: 2026-10-27

## Decision

Local-state owns versioned gameplay baselines captured inside participant creation,
after source-derived initial HP is finalized. Baselines contain HP provenance,
variant, mortality policy/state, initiative and saved conditions/resources;
participant identity, assigned level and editorial metadata stay live. The existing
participant hidden flag now drives only roster dimming; it does not affect turn
eligibility, initiative or encounter difficulty. Existing participants have
no inferred baseline. Source-backed declarations remain authoritative reference
facts and are not copied into mutable condition rows.

App-service owns the distinct reset and eight-hour overnight policies and projects
consequences before submission. The browser collects selected participants and
explicit healing amounts or a full-HP policy, without reimplementing recovery arithmetic. A full
encounter revision guards the complete snapshot; source-derived rest plans also
guard the reference revision. One local-state savepoint writes all replacements,
invalidates superseded decisions and saves a request identity. Identical retries
recover before guards, preventing repeated healing/reduction; intent reuse fails.
Historical roll/event receipts survive reset so old retries cannot reapply events.

Whole reset returns to Draft/round 1 and releases all boundary claims. Participant
reset preserves the encounter's round and ends its active turn only when that
participant owns it. Rest ends the active turn while preserving encounter status,
round and initiative. Outgoing turn snapshots include the roster and are cleared
by either operation. Turn allocations are cleared when their turn ends.
Ending the turn also clears encounter boundary claims so restarting the retained
round can allocate actions and process remaining conditions without stale claims.

## Consequences

Missing creation facts are visible and block a whole reset instead of producing a
partial success. Re-adding a legacy participant captures a fresh baseline. Reset
never means healing or re-reading the latest source HP. Overnight recovery follows
current source facts, preserves source-controlled conditions and exposes unknown
HP inputs for table resolution. Equipment inventory, general Foundry rule
execution, and calendar scheduling are separate scopes.
