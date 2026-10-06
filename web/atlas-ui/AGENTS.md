# Atlas UI Agent Guidance

This package owns the React browser frontend for PF2e Atlas. Follow these rules when editing files under `web/atlas-ui`.

## Frontend Ownership

- Keep product semantics in Rust app-service/app-model contracts. The frontend should render generated DTOs and call the thin API client; it should not duplicate retrieval, filtering, ranking, record projection, saved-list, or encounter product rules.
- Use Ant Design as the component library for generic application UI. Prefer Ant components for buttons, icon buttons, links, navigation controls, forms, tables, modals, popovers, tooltips, selectors, checkboxes, alerts, empty/loading states, tags, and pagination before adding custom controls.
- Hand-roll UI only when the surface is product-specific, such as record presentation, PF2e stat/rendering layouts, rich result summaries that Ant list/table primitives cannot express cleanly, or encounter-runtime panels.
- Use Atlas theme tokens and existing CSS variables. Do not add ad hoc raw colors or one-off palettes. Derive custom Atlas state variables from Ant semantic tokens, not raw brand tokens such as `colorPrimary`, where the variable represents selected, active, link, hover, or focus behavior.

## Shared UI Primitives

Before creating a feature-local interaction pattern, check `src/shared/ui`.

Before implementing any generic control, overlay, popup, or confirmation, search both
`src/shared/ui` and existing Ant Design usage for an established pattern. Destructive
buttons and confirmations must use `DangerActionButton` or `useConfirmDangerAction` when
their contracts fit; do not replace them with feature-local modal state or popup CSS.
A deviation requires a documented product-specific reason explaining why the existing
Ant or shared primitive cannot satisfy the interaction.

Ant overlays must be created through the application context (for example,
`App.useApp().modal`) or a shared context-aware wrapper. Do not call static overlay APIs
such as `Modal.confirm`, `message`, or `notification` directly: they render outside the
configured provider and can lose Atlas theme, locale, and token context.

- Use `IndexTable` for clickable entity/index tables.
- Use `EntityIndexPage` for `/things` index pages with a title, summary, actions, and table body.
- Use `EditableCommitField` for inline fields that keep a local draft and commit on blur or Enter.
- Use `SearchPickerModal` for modal record-search selection flows.
- Use `PaneIconButton` and `PaneIconLink` for pane-header icon actions.
- Use `DangerActionButton` for buttons that open a destructive confirmation before running the action.
- Keep lower-level hooks such as `useConfirmDangerAction` for non-button callbacks where a shared button is not the right shape.

If a second surface needs a behavior, move it into `src/shared/ui` instead of copying the feature-local implementation. Feature modules should compose shared primitives with feature-specific requests, labels, mutations, and DTOs. Shared primitives should normally wrap or compose Ant components; do not create plain-button or plain-anchor replacements for generic Ant behavior unless there is a documented product-specific reason.

## CSS Layout

- Put reusable component styles under `src/styles/ui`.
- Put feature-specific styles under the owning feature stylesheet.
- Keep page-level structure unframed unless the existing shared layout primitive intentionally frames it.
- Avoid nested cards and avoid feature-local copies of pane, table, modal, popover, and index-page layout patterns.
- Prefer Ant component tokens for generic interaction states, such as selected table rows, link text, hover fills, active navigation, and disabled/loading treatment. Custom CSS should use semantic Atlas variables for product-specific layouts rather than mixing `--accent` into state colors directly.

## Validation

Run frontend validation for UI changes:

```bash
npm --prefix web/atlas-ui run verify
```

For narrow refactors, also run focused tests for the touched surface before the full verify.

Frontend implementers must validate the matching candidate with Safari MCP in a separate tab after affected automated checks, inspecting desktop1440, intermediate1024 and narrow390 layouts, dark/light themes and keyboard interactions. Fix obvious in-scope defects before independent review. Record exact candidate/URL, screenshots, interactions and limitations. Safari MCP is independent of the AO Browser panel: do not wait for AO Browser capabilities or substitute reviewer screenshots. Preserve unrelated tabs, protected runtimes and saved state. Browser validation grants no runtime/build/publication authority.
