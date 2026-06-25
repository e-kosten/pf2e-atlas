# Frontend Guidelines

The Atlas web UI is a React frontend over generated Rust app DTOs. It uses Ant Design for base controls and a small first-party shared UI layer for Atlas-specific interaction patterns.

## Boundaries

- `web/atlas-ui` owns browser presentation, local browser state, URL state, API calls, and component composition.
- Rust `atlas-app-model` and `atlas-app-service` own product DTOs and workflow semantics.
- Frontend code should not recreate filter catalogs, retrieval semantics, saved-list behavior, encounter mutation rules, or record presentation contracts that already come from the backend.

## Shared UI Layer

Use `src/shared/ui` for repeated Atlas interaction primitives over Ant Design. Current shared primitives include:

- `tables/IndexTable.tsx`: clickable, keyboard-activatable index tables.
- `pages/EntityIndexPage.tsx`: common index page structure for title, summary, actions, and table content.
- `forms/EditableCommitField.tsx`: inline draft fields that commit on blur or Enter and revert on Escape.
- `pickers/SearchPickerModal.tsx`: modal record search and selection flow with debounced backend search.
- `actions/PaneAction.tsx`: pane-header icon buttons and links.
- `actions/DangerActionButton.tsx`: destructive button plus confirmation modal.
- `actions/confirmDangerAction.tsx`: confirmation helper for cases that are not naturally buttons.

Feature modules should pass feature-specific labels, requests, mutations, and DTOs into these primitives. They should not copy the primitive behavior into local components.

## Ant Design

Ant Design is the selected component library. Prefer native Ant Design components for common controls:

- `Table` for table mechanics, usually through `IndexTable` for entity indexes.
- `Form`, `Input`, `InputNumber`, `Select`, `Checkbox`, and `Button` for forms and edits.
- `Modal` and `Popover` for overlays.
- Ant Design loading, disabled, empty, and error affordances where they fit.

Wrap or compose Ant Design only when Atlas has a repeated product interaction that should behave consistently across surfaces.

## Styling

- Use Atlas CSS variables and Ant Design theme tokens; avoid raw color values.
- Put shared primitive styles under `src/styles/ui`.
- Keep feature-specific styles in the feature stylesheet.
- Do not create nested cards or duplicate pane/index/table styles in feature modules.
- Prefer stable layout dimensions for repeated controls so dynamic labels, icons, loading states, and validation text do not shift the page.

## Validation

Use the package gate for frontend changes:

```bash
npm --prefix web/atlas-ui run verify
```

For refactors, run focused tests for touched surfaces first, then the full verify.
