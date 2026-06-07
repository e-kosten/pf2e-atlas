# Rust Web UI Architecture Review

Status: proposed
Priority: soon
Owner: unassigned
Last reviewed: 2026-06-07

## Problem

The first Atlas web UI vertical slice now has enough real behavior to review as an architecture surface instead of as a throwaway prototype. It includes generated DTO consumption, a thin API client, TanStack Query server state, URL/search state, reducer-backed workspace interaction state, Ant Design composition, dynamic filters, result windows, record detail loading, and local styling.

Without a focused architecture review, follow-up feature work could grow around accidental component structure, mixed state ownership, or frontend-local semantics that should stay in Rust app-service/search layers.

## Desired Outcome

Run a focused architecture review of `web/atlas-ui` and its app-layer boundaries before adding substantial new capabilities.

The review should assess:

- whether frontend code uses generated Rust DTOs rather than duplicating app contracts;
- whether API client code remains thin transport glue over `atlas-web`;
- whether TanStack Query, URL state, and local reducer state have clear ownership;
- whether Ant Design components are wrapped/composed in a way that preserves Atlas product semantics;
- whether result-window, record-detail, and filter-discovery behavior avoids leaking backend implementation details;
- whether module layout and tests are strong enough for the next feature slices.

## Constraints

- Treat this as review and follow-through planning, not a feature implementation.
- Do not move retrieval, discovery, ranking, or record-presentation semantics into frontend code.
- Prefer direct cleanup over compatibility shims if the review finds transitional frontend structure.

## Related

- [Architecture overview](../../architecture/overview.md)
- [ADR 0029: Local web app boundary](../../architecture/decisions/0029-local-web-app-boundary.md)
- [Component library decision](../../../web/atlas-ui/docs/component-library-evaluation.md)
