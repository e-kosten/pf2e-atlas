# Category Relevance Script

Status: proposed  
Priority: later  
Owner: unassigned  
Last reviewed: 2026-04-20

## Problem

Large tagging passes benefit from working in coherent batches, but there is not yet tooling that can quickly surface the relevant tag set for a shared family or cohort and complain when a batch mixes incompatible scopes.

## Desired Outcome

Add tooling that can:

- accept a family or bounded batch of records
- report the relevant tag set for that batch
- warn when the input does not share the same effective tag space
- support more scalable orchestration for large review or assignment runs

## Constraints

- The tool should support better batching without suppressing discovery of genuinely missing categories.
- It should help structure review work rather than becoming an authoritative replacement for editorial judgment.

## Related

- [Editorial architecture](../../architecture/node/editorial.md)
