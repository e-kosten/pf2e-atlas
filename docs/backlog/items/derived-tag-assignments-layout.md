# Derived-Tag Assignments Layout

Status: proposed  
Priority: soon  
Owner: unassigned  
Last reviewed: 2026-04-20

## Problem

The assignments area does not yet have a durable storage layout designed for a very large authored corpus. As assignment coverage grows toward the full live record set, an ad hoc layout will become hard to navigate and maintain.

## Desired Outcome

Choose and document a directory/file structure for authored assignments that:

- scales to a very large record count
- stays understandable to human editors
- supports predictable tooling and review workflows
- aligns with how the rest of the tag editorial system is organized

## Constraints

- The structure should be durable enough that future migrations are rare.
- Human editability matters; this is not just a machine-generated cache.
- The chosen shape should fit the repo’s existing authored/runtime/editorial split.

## Related

- [Editorial architecture](../../architecture/editorial.md)
