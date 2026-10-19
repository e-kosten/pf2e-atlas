# Atlas UI Agent Guidance

This package owns the React browser frontend for PF2e Atlas. Follow these rules when editing files under `web/atlas-ui`.

## Frontend Ownership

- Keep product semantics in Rust app-service/app-model contracts. The frontend should render generated DTOs and call the thin API client; it should not duplicate retrieval, filtering, ranking, record projection, saved-list, or encounter product rules.
- Use Ant Design as the component library for generic application UI. Prefer Ant components for buttons, icon buttons, links, navigation controls, forms, tables, modals, popovers, tooltips, selectors, checkboxes, alerts, empty/loading states, tags, and pagination before adding custom controls.
- Hand-roll UI only when the surface is product-specific, such as record presentation, PF2e stat/rendering layouts, rich result summaries that Ant list/table primitives cannot express cleanly, or encounter-runtime panels.
- Use Atlas theme tokens and existing CSS variables. Do not add ad hoc raw colors or one-off palettes. Derive custom Atlas state variables from Ant semantic tokens, not raw brand tokens such as `colorPrimary`, where the variable represents selected, active, link, hover, or focus behavior.

Follow the root `AGENTS.md` Product UI Copy guidance for both frontend text and
backend-projected text. Review the rendered result, not just the component code.

## Shared UI Primitives

Before creating a feature-local interaction pattern, check `src/shared/ui`.

- Use `PrimaryToolbar` for wrapping primary control rows. Inherit default Ant sizes; reserve small controls for dense inline/table actions. Keep search flexible and neighboring controls top-aligned when selected choices wrap.
- Use `IndexTable` for clickable entity/index tables.
- Use `EntityIndexPage` for `/things` index pages with a title, summary, actions, and table body.
- Use `EditableCommitField` for inline fields that keep a local draft and commit on blur or Enter.
- Use `SearchPickerModal` for modal record-search selection flows.
- Use `PaneIconButton` and `PaneIconLink` for pane-header icon actions.
- Use `DangerActionButton` for buttons that open a destructive confirmation before running the action.
- Keep lower-level helpers such as `useConfirmDangerAction` for non-button callbacks where a shared button is not the right shape.

If a second surface needs a behavior, move it into `src/shared/ui` instead of copying the feature-local implementation. Feature modules should compose shared primitives with feature-specific requests, labels, mutations, and DTOs. Shared primitives should normally wrap or compose Ant components; do not create plain-button or plain-anchor replacements for generic Ant behavior unless there is a documented product-specific reason.

For overlays and secondary tasks, follow
[Choosing an interaction surface](docs/frontend-guidelines.md#choosing-an-interaction-surface).
For record browsing, links and nested previews, follow
[In-place reference reading](docs/frontend-guidelines.md#in-place-reference-reading).
Reuse the shared browse/reader and preview owners with exact backend navigation;
keep feature selection and mutations separate. Do not invent feature-local
Popover/Drawer or link-navigation policies.

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

Follow [responsive acceptance](docs/frontend-guidelines.md#responsive-design-and-acceptance)
for layout, navigation, overlays and forms. Run `npm --prefix web/atlas-ui run
test:devices` when those shared behaviors change; it uses a small real-app fixture.
Keep Astra visual/interaction review separate from automated checks. Test actual
taps in touch contexts and distinguish viewport resizing, emulation and physical
device evidence. Report unavailable device/zoom/software-keyboard checks explicitly.

Human product previews use the existing full-corpus artifact and verified query
models, with every stored profile usable. Reuse a compatible running backend for
frontend-only changes. Reserve the model-free fixture for automated acceptance
and targeted fault checks; follow CONTRIBUTING.md for preview/state ownership.

Contextual feedback uses the root Ant `App` inside `ConfigProvider`. Destructive
button and callback flows use `useConfirmDangerAction`; prepared source dialogs
use contextual `Modal.useModal`. Do not use static Ant feedback methods. Style
Ant internals through documented `classNames`, `styles` and component tokens;
Atlas-owned classes describe product layout rather than internal Ant DOM.

Follow the root repository early functional UX checkpoint rule for substantial visible frontend changes. Keep existing tests enabled and disclose stale expectations; complete frontend gates and Astra visual review after the user confirms the direction.
