# Atlas UI Agent Guidance

This package owns the React browser frontend for PF2e Atlas. Follow these rules when editing files under `web/atlas-ui`.

## Frontend Ownership

- Keep product semantics in Rust app-service/app-model contracts. The frontend should render generated DTOs and call the thin API client; it should not duplicate retrieval, filtering, ranking, record projection, saved-list, or encounter product rules.
- Use Ant Design as the component library. Prefer Ant Design components for forms, tables, modals, popovers, selectors, checkboxes, alerts, and loading states before adding custom controls.
- Use Atlas theme tokens and existing CSS variables. Do not add ad hoc raw colors or one-off palettes.

## Shared UI Primitives

Before creating a feature-local interaction pattern, check `src/shared/ui`.

- Use `IndexTable` for clickable entity/index tables.
- Use `EntityIndexPage` for `/things` index pages with a title, summary, actions, and table body.
- Use `EditableCommitField` for inline fields that keep a local draft and commit on blur or Enter.
- Use `SearchPickerModal` for modal record-search selection flows.
- Use `PaneIconButton` and `PaneIconLink` for pane-header icon actions.
- Use `DangerActionButton` for buttons that open a destructive confirmation before running the action.
- Keep lower-level helpers such as `confirmDangerAction` for non-button callbacks where a shared button is not the right shape.

If a second surface needs a behavior, move it into `src/shared/ui` instead of copying the feature-local implementation. Feature modules should compose shared primitives with feature-specific requests, labels, mutations, and DTOs.

## CSS Layout

- Put reusable component styles under `src/styles/ui`.
- Put feature-specific styles under the owning feature stylesheet.
- Keep page-level structure unframed unless the existing shared layout primitive intentionally frames it.
- Avoid nested cards and avoid feature-local copies of pane, table, modal, popover, and index-page layout patterns.

## Validation

Run frontend validation for UI changes:

```bash
npm --prefix web/atlas-ui run verify
```

For narrow refactors, also run focused tests for the touched surface before the full verify.
