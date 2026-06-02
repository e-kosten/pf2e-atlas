# Rust Ratatui Workbench

Status: planned
Priority: soon
Owner: unassigned
Last reviewed: 2026-05-19

## Problem

PF2e Atlas has a strong CLI and agent skill surface, but it does not yet have an interactive terminal workbench over the Rust runtime. The workbench should preserve the useful product ideas from the previous terminal backlog without carrying over implementation-specific assumptions.

## Desired Outcome

Build a Ratatui workbench over the Rust runtime crates for search, browse, detail reading, filter exploration, graph context, and future editorial workflows.

The workbench should cover:

- search result list/detail navigation
- record detail pages with readable kind-specific facts
- reference navigation and back-stack behavior
- filter/schema exploration and structured query construction
- action palette or command menu behavior with fast keyboard selection
- modal/prompt placement rules that fit terminal workflows
- list viewport behavior, pointer hit testing, and mouse text selection where terminal capability supports it
- shared interaction contracts for lists, menus, editors, workspace panes, and detail pages
- render tests with Ratatui `TestBackend`
- manual acceptance checks for common terminal environments

## Constraints

- Compose through `atlas-runtime`, `atlas-search`, `atlas-index`, and `atlas-record`; do not open SQLite or embedding models directly from screen code.
- Keep durable state models separate from rendered row/view models.
- Treat terminal capability checks for clipboard, links, and pointer behavior as explicit abstractions.
- Do not recreate structures from retired terminal implementations.

## Notes

This item preserves the useful intent from retired terminal backlog items such as search interaction cleanup, view/detail presentation, query editor shortcuts, page navigation, shared UI model boundaries, pointer target hit testing, and mouse text selection.

## Related

- [Runtime architecture](../../architecture/runtime.md)
- [ADR 0026: Rust CLI product surface](../../architecture/decisions/0026-rust-cli-product-surface.md)
