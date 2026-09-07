# Frontend Guidelines

The Atlas web UI is a React frontend over generated Rust app DTOs. It uses Ant Design for general application UI and a small first-party shared UI layer for Atlas-specific interaction patterns.

## Boundaries

- `web/atlas-ui` owns browser presentation, local browser state, URL state, API calls, and component composition.
- Rust `atlas-app-model` and `atlas-app-service` own product DTOs and workflow semantics.
- Frontend code should not recreate filter catalogs, retrieval semantics, saved-list behavior, encounter mutation rules, or record presentation contracts that already come from the backend.
- Creature and hazard record/encounter components consume their generated named, typed DTOs directly. Do not introduce generic section/value bags, encoded-target lookup, fallback presentation, or client-side joins to recover mechanics. Hazard static and runtime payloads are composed by the backend; image paths are not fetched or displayed, and license metadata appears only in explicit source provenance. Unavailable presentations remain an explicit boundary for families whose separately owned migrations have not landed.
- Hazard detail uses its own `HazardDefensesPanel`: HP/current/maximum/temporary composition and derived broken threshold remain hazard-owned, while only scalar stat rhythm, signed save formatting, IWR rows, and note typography are shared with the creature surface. Detection and disable precede defenses and activities; authored routine and reset behavior belong to the following Operation section. Strike mode and the derived one-action glyph consume the app service's optional typed activity fields; the component never infers either from traits, labels, or provenance. Reviewed source-only hazard facts may appear only in the secondary typed provenance disclosure, without parsing exact JSON or source paths.
- Creature, hazard, spell, and ritual detail share one outer issue and reference supplement. Typed outer issues render once there rather than through family-local availability warnings. Human record links come only from outer graph references; opening the detail disclosure deliberately requests both directions through the existing record-detail limits, while compact and encounter surfaces do not initiate graph reads. Internal entity or occurrence membership, authored relationship fallbacks, and source locators are never ordinary link or display labels; explicit source provenance remains a separate secondary disclosure.
- Standalone spell and ritual detail consumes `SpellSurfaceView` directly. The page presents one backend-resolved definition in Casting, Range & targets, Effect, and Heightening order, with the compact form catalog and cast-rank controls integrated above that same definition. Ritual requirements and supported rule explanations are secondary disclosures rather than a duplicate stat block. The browser preserves catalog order, submits an opaque form ID with the requested rank through the existing record-detail endpoint, and replaces the main regions only with a matching returned `effective_form`; while a request is pending, fails, or is wholly unavailable, the last valid result stays visible with an explicit status. Effective classification is the sole rank and trait owner, while the typed body family owns the Spell or Ritual label. The browser never falls back to generic outer metadata, receives or applies authored patches, calculates heightening, parses form labels or IDs, exposes internal rule or damage identities as labels, substitutes numeric query range for authored range text, or turns a definition into encounter casting state.

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

## Record readability and validation

Spell details prioritize selected-form damage and core casting facts, with compact upper-right form/rank controls that wrap below the heading at constrained widths. Mechanics use the full pane width without a floating summary card. A sole Base form is static. Description retains authored spell prose. Typed damage category, effect type, materials and spellcasting ability modifier remain visible beside the effect; known false values remain explicit. Pending Apply uses a reserved inline status and aria-busy while preserving the controls and last valid mechanics. Background record refetch keeps the last valid selected form mounted; selected responses must match record, form and rank, and reference responses must match record and both requested limits. Typed unavailable/error responses retain the last valid selected mechanics while exposing recovery.

Search moves filters into an Ant Drawer at1100px and below, retains query and filter counts, and switches between results/detail at760px and below. Links, focus indicators and primary control text use theme semantic tokens and must be checked in both themes. References use server-provided next limits and expose the terminal50-record cap honestly. Issue grouping preserves each distinct typed fact and renders server-owned subject/component labels. Foundry hearing/runtime detection settings appear only in explicit source provenance, never in ordinary hazard mechanics or explanatory copy.

Use Safari MCP in a separate tab to inspect the exact candidate at1440,1024 and390px, dark/light and keyboard navigation, before independent review. Fix obvious in-scope defects and preserve screenshot/interaction evidence bound to candidate bytes and URL. No AO Browser wait or alternate-runtime screenshot substitutes for this gate.

Reference-search navigation carries the generated relationship constraint and clears free text. Incoming means records referencing the selected entity; outgoing means records referenced by it. A removable chip uses the exact keyed server label or an honest unavailable/missing state, never a raw key or name-based query. Result windows own complete filtered counts and pagination; the references panel limit is not the search result scope. Preserve the relationship during other filter edits/discovery and URL restoration; abort superseded searches and never attach their window IDs to another constraint.

Hazard structural statistics consume the server-computed applicability state. Hide AC/HP/hardness/BT only for `inapplicable`; keep supported values for `unknown` and `applicable`. Never derive this state from complexity, zeros, raw source flags or prose. Preserve independent saving throws, IWR and HP notes.
