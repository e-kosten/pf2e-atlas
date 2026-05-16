# Typed Seams Cleanup

Status: proposed  
Priority: later  
Owner: unassigned  
Last reviewed: 2026-04-21

## Problem

The next useful typing work is not just “enable more strict rules.” There are a few code seams that still make type-aware linting slower, noisier, and more assertion-heavy than necessary.

The original scratch note called out several concrete hotspots:

- metadata field catalog access relying on stringly typed lookups and repeated type recovery
- prompt result flows where sentinel values and narrowing still create friction
- matcher code depending on awkward external-library typing behavior
- a large remaining typed-lint backlog in test helpers and mocks

## Desired Outcome

Continue type-safety work as a set of bounded seam cleanups rather than as a broad “turn on more strictness” campaign.

That work should:

- expose stronger typed helpers around the metadata field catalog
- simplify prompt result shapes where sentinel handling still forces repeated assertions
- isolate external-library typing quirks behind small adapters with focused tests
- tighten test fixtures and mocks before taking on broader unsafe-any style typed-lint passes

## Constraints

- Prefer bounded, high-signal typing passes over broad rule flips.
- Do not let typing cleanup silently change runtime behavior in fragile seams like matcher adapters.
- Keep the next pass scoped enough that validation remains credible.

## Notes

The original scratch note suggested taking `@typescript-eslint/require-await` as a bounded next typed-lint pass instead of jumping directly to the broader unsafe-any family.

## Related

- [Architectural boundaries](../../architecture/node/boundaries.md)
