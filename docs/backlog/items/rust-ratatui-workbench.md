# Rust Ratatui Workbench

Status: planned
Priority: soon
Owner: unassigned
Last reviewed: 2026-10-13

## Problem

PF2e Atlas needs an interactive terminal workbench for keyboard-driven discovery,
reference reading and preparation over its shared application contracts.

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

- Compose product workflows through `atlas-app-service` and `atlas-app-model`,
  which use the shared runtime and retrieval owners. Screen code does not open
  SQLite or embedding models directly.
- Keep durable state models separate from rendered row/view models.
- Treat terminal capability checks for clipboard, links, and pointer behavior as explicit abstractions.
- Define terminal-specific navigation and rendering around those contracts.

## Related

- [Runtime architecture](../../architecture/runtime.md)
- [ADR 0026: Rust CLI product surface](../../architecture/decisions/0026-rust-cli-product-surface.md)
