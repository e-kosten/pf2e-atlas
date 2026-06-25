# Rust Web UI Frontend Architecture Cleanup

Status: done  
Priority: soon  
Owner: unassigned  
Last reviewed: 2026-06-25

## Problem

This item is complete. The Atlas web frontend had grown beyond the initial prototype shape, with route files and feature components combining route composition, TanStack Query orchestration, mutation request construction, form helpers, and presentation details.

## Desired Outcome

The landed outcome is:

- `web/atlas-ui/src/app` owns app shell, route parsing, and top-level route composition.
- `web/atlas-ui/src/features` owns feature-specific search, saved-list, record, and encounter UI modules.
- `web/atlas-ui/src/shared` owns cross-surface filter, layout, record presentation, record preview, and theme primitives.
- saved-list filtering and search filtering reuse shared filter discovery orchestration.
- encounter participant edit helpers have one tested frontend owner.
- record detail loading and record preview popovers are shared instead of route-local.
- Ant Design and prototype-era names no longer appear in stable product module names.
- theme tokens have one clear frontend owner, with CSS values treated as runtime variables and fallback defaults.

## Constraints

- Retrieval, ranking, filter discovery semantics, and record contract ownership remain outside frontend code.
- App DTOs continue to flow through the generated binding aggregation file.
- The refactor landed as direct replacement without compatibility shims or mixed old/new frontend paths.
- Cross-surface layout and overlay primitives are shared unless the interaction is genuinely feature-specific.

## Related

- [Architecture overview](../../../architecture/overview.md)
- [Runtime architecture](../../../architecture/runtime.md)
- [ADR 0029: Local web app boundary](../../../architecture/decisions/0029-local-web-app-boundary.md)
- [Rust web UI architecture review](../../items/rust-web-ui-architecture-review.md)
- [Component library decision](../../../../web/atlas-ui/docs/component-library-evaluation.md)
