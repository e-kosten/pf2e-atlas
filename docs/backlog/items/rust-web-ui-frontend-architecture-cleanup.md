# Rust Web UI Frontend Architecture Cleanup

Status: proposed
Priority: soon
Owner: unassigned
Last reviewed: 2026-06-25

## Problem

The first Atlas web UI surfaces have grown beyond the initial prototype shape. The frontend still respects the Rust app-service boundary, but several route files and feature components now combine route composition, TanStack Query orchestration, mutation request construction, form helpers, and presentation details.

This creates repeated frontend-local behavior, especially around encounter participant updates and filter discovery, and makes future UI surfaces more likely to copy existing orchestration instead of reusing a deliberate frontend architecture.

## Desired Outcome

Refactor `web/atlas-ui` into a clearer feature-oriented frontend structure where:

- route components are thin shells over feature hooks and pane composition;
- shared filter discovery orchestration is reused by search, saved lists, and future filtered surfaces;
- encounter participant edit helpers have one tested frontend owner;
- record detail loading, pane presentation primitives, and record preview popovers are shared instead of route-local;
- Ant Design remains an implementation detail rather than appearing in stable product module names;
- theme tokens have one clear source of truth, with CSS fallback values treated as boot defaults only.

## Constraints

- Do not move retrieval, ranking, filter discovery semantics, or record contract ownership into frontend code.
- Continue importing app DTOs through the generated binding aggregation file.
- Prefer direct replacement over compatibility shims or mixed old/new paths.
- Keep behavior stable; this is a structure and ownership cleanup, not a redesign.
- Put cross-surface layout and overlay primitives in shared frontend modules unless the interaction is genuinely feature-specific.

## Related

- [Architecture overview](../../architecture/overview.md)
- [Runtime architecture](../../architecture/runtime.md)
- [ADR 0029: Local web app boundary](../../architecture/decisions/0029-local-web-app-boundary.md)
- [Rust web UI architecture review](./rust-web-ui-architecture-review.md)
- [Component library decision](../../../web/atlas-ui/docs/component-library-evaluation.md)
