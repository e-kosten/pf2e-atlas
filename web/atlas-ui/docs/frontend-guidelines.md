# Frontend Guidelines

The Atlas web UI is a React frontend over generated Rust app DTOs. It uses Ant Design for general application UI and a small first-party shared UI layer for Atlas-specific interaction patterns.

## Boundaries

- `web/atlas-ui` owns browser presentation, local browser state, URL state, API calls, and component composition.
- Rust `atlas-app-model` and `atlas-app-service` own product DTOs and workflow semantics.
- Frontend code should not recreate filter catalogs, retrieval semantics, saved-list behavior, encounter mutation rules, or record presentation contracts that already come from the backend.
- Creature and hazard record/encounter components consume their generated named, typed DTOs directly. Do not introduce generic section/value bags, encoded-target lookup, fallback presentation, or client-side joins to recover mechanics. Hazard static and runtime payloads are composed by the backend; image paths are not fetched or displayed, and license metadata appears only in explicit source provenance. Unavailable presentations remain an explicit boundary for families whose separately owned migrations have not landed.
- Standalone spell and ritual detail consumes `SpellSurfaceView` directly. The browser preserves the backend form catalog order, submits an opaque form ID together with a cast-rank choice through the existing record-detail endpoint, and renders only the matching returned `selected_form`. It never applies patches, calculates heightening, parses form labels or IDs, substitutes the numeric range query projection for authored range text, or turns a definition into encounter casting state.

## Shared UI Layer

Use `src/shared/ui` for repeated Atlas interaction primitives over Ant Design. Shared primitives should normally compose Ant components instead of recreating generic control behavior with plain elements and local CSS. Current shared primitives include:

- `tables/IndexTable.tsx`: clickable, keyboard-activatable index tables.
- `pages/EntityIndexPage.tsx`: common index page structure for title, summary, actions, and table content.
- `forms/EditableCommitField.tsx`: inline draft fields that commit on blur or Enter and revert on Escape.
- `pickers/SearchPickerModal.tsx`: modal record search and selection flow with debounced backend search.
- `actions/PaneAction.tsx`: pane-header icon buttons and links.
- `actions/DangerActionButton.tsx`: destructive button plus confirmation modal.
- `actions/confirmDangerAction.tsx`: confirmation helper for cases that are not naturally buttons.

Feature modules should pass feature-specific labels, requests, mutations, and DTOs into these primitives. They should not copy the primitive behavior into local components.

## Ant Design

Ant Design is the selected component library. Generic application UI should use Ant components and component tokens before adding custom controls or local state styling. This keeps hover, focus, selected, disabled, loading, contrast, and light/dark behavior in the same design system.

- `Table` for table mechanics, usually through `IndexTable` for entity indexes.
- `Form`, `Input`, `InputNumber`, `Select`, `Checkbox`, and `Button` for forms and edits.
- `Button`, including `icon`, `type="text"`, `type="link"`, `danger`, and loading states, for generic actions.
- `Typography.Link` or Ant `Button type="link"` for link-colored actions.
- `Menu`, `Tabs`, or `Segmented` for generic navigation or mode selection.
- `Modal`, `Popover`, `Tooltip`, `Alert`, `Empty`, `Tag`, and `Pagination` for standard application states.
- Ant Design loading, disabled, empty, and error affordances where they fit.

Wrap or compose Ant Design when Atlas has a repeated product interaction that should behave consistently across surfaces. Keep custom Atlas components for product-specific surfaces such as record presentation, PF2e stat/rendering layouts, rich result summaries when Ant list/table primitives cannot express the layout cleanly, and encounter-runtime panels. Do not hand-roll generic buttons, icon buttons, links, navigation controls, table selection states, overlays, or form controls when an Ant component exists.

When styling Ant components, prefer `ConfigProvider` component tokens such as `Table.rowSelectedBg` and semantic global tokens such as `colorPrimaryBg`, `colorPrimaryBgHover`, `colorPrimaryBorder`, `colorPrimaryText`, `colorLink`, and `controlItemBgActive`. Avoid using `colorPrimary` directly as a foreground text color or low-opacity highlight fill.

## Styling

- Use Atlas CSS variables and Ant Design theme tokens; avoid raw color values.
- Atlas CSS variables should describe semantic app states, such as active background, active border, active text, muted text, and panel surfaces. They should be derived from Ant semantic tokens rather than raw brand tokens where possible.
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
