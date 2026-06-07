# Rust Web Keyboard Navigation And Focus

Status: proposed
Priority: soon
Owner: unassigned
Last reviewed: 2026-06-07

## Problem

The Atlas web UI should support a keyboard-driven workflow for users who prefer vim-like navigation or repeated search/browse actions. The initial slice has basic result movement, but the broader focus model, shortcut set, and pane navigation behavior are not yet designed as a coherent interaction contract.

Without a focused pass, keyboard behavior can become a set of local handlers that conflict with browser defaults, Ant Design controls, editable inputs, and future detail/preview workflows.

## Desired Outcome

Define and implement a coherent keyboard navigation and focus model for the web UI.

The pass should cover:

- result list navigation with standard arrows and vim-style keys where appropriate;
- focus movement between search, filters, results, and detail panes;
- safe behavior while text inputs, selects, and editable controls are focused;
- open/select/detail/back behavior for record navigation;
- visible focus affordances and accessibility expectations;
- shortcut documentation or discoverability if needed;
- tests for keyboard behavior that is likely to regress.

## Constraints

- Prefer browser-idiomatic accessibility and Ant Design control behavior over forcing terminal-style semantics everywhere.
- Keep keyboard behavior product-focused and avoid implementing a full TUI model in the browser.
- Do not let local keyboard handlers bypass URL/detail state ownership.

## Related

- [Architecture overview](../../architecture/overview.md)
- [ADR 0029: Local web app boundary](../../architecture/decisions/0029-local-web-app-boundary.md)
- [ADR 0006: Shared TUI interaction contracts](../../architecture/decisions/0006-shared-tui-interaction-contracts.md)
