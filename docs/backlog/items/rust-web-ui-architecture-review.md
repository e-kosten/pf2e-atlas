# Record Presentation and UI Architecture Review

Status: proposed
Priority: soon
Owner: unassigned
Last reviewed: 2026-10-10

## Problem

The first Atlas web UI vertical slice now has enough real behavior to review as an architecture surface instead of as a throwaway prototype. It includes generated DTO consumption, a thin API client, TanStack Query server state, URL/search state, reducer-backed workspace interaction state, Ant Design composition, dynamic filters, result windows, record detail loading, and local styling.

Without a focused architecture review, follow-up feature work could grow around accidental component structure, mixed state ownership, or frontend-local semantics that should stay in Rust app-service/search layers.

## Desired Outcome

Review presentation from ingest upward before adding substantial new capabilities:
checked DTOs and content preparation, borrowed source views, app contracts, then
CLI and `web/atlas-ui` consumers. The coordinated artifact cutover may retain an
imperfect presentation shape; family-specific UI modeling belongs in this
follow-up rather than expanding that cutover's scope.

Inspect `integration/record-refactor`
([PR 7](https://github.com/e-kosten/pf2e-atlas/pull/7)) for feature requirements and
useful product ideas. Its implementation is inspiration, not evidence of architectural or
modeling correctness. Evaluate retained and new abstractions from first principles,
including whether they do too much and whether maintained libraries can replace
custom logic.

The review should assess:

- whether frontend code uses generated Rust DTOs rather than duplicating app contracts;
- whether API client code remains thin transport glue over `atlas-web`;
- whether TanStack Query, URL state, and local reducer state have clear ownership;
- whether Ant Design components are wrapped/composed in a way that preserves Atlas product semantics;
- whether result-window, record-detail, and filter-discovery behavior avoids leaking backend implementation details;
- whether module layout and tests are strong enough for the next feature slices.

Review the shared CLI/web presentation boundary as well as React components.
`RecordSurfaceView` currently groups source-backed facts into generic sections,
value groups and activities. Assess whether that contract supports useful
family-specific layouts or unnecessarily dictates them. Encounter views also
need scrutiny of retained presentation vocabulary and explicit unavailable facts.
Treat these as consumer projections that can change, not a second authored model.

The borrowed source accessors preserve checked DTO values without retaining
another creature or spell body. Review their grouping and naming against actual
consumer needs; presentation accessors extending `ActorQueryView` may warrant a
broader borrowed actor view. Avoid expanding source wrappers speculatively or
duplicating extraction in each renderer. Revisit these contracts before substantial
UI and CLI presentation expansion; no stored-DTO redesign is implied.

## Constraints

- Treat this as review and follow-through planning, not a feature implementation.
- Do not move retrieval, discovery, ranking, or record-presentation semantics into frontend code.
- Prefer direct cleanup over compatibility shims if the review finds transitional frontend structure.

## Related

- [Architecture overview](../../architecture/overview.md)
- [ADR 0029: Local web app boundary](../../architecture/decisions/0029-local-web-app-boundary.md)
- [Component library decision](../../../web/atlas-ui/docs/component-library-evaluation.md)
