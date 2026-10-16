# Shared UI consistency and saved-list presentation

Status: done
Last reviewed: 2026-10-16

Delivered through [PR58](https://github.com/e-kosten/pf2e-atlas/pull/58): shared
list/search browsing, consistent indexes/toolbars, readable content widths,
list editing/Add and in-place reference reading. User visual approval,
independent review, Astra browser review and hosted CI preceded merge.
Repeatable device/browser acceptance continues in
[responsive UI validation](../../items/responsive-ui-validation.md).

## Problem

Saved-list views have not received the same design attention as search, reference
reading and encounters. The application needs one coherent UI language and a
deliberate shared component set across all product surfaces.

## Intended work

Review saved-list index and detail views against the current application chrome,
search browser, record reader and encounter workflows. Inventory existing shared
components and identify duplicated or inconsistent implementations before editing.
Standardize layout, spacing, typography, control sizes, action placement, result
selection/preview, loading/empty/error states and responsive behavior where the
interaction serves the same purpose. Preserve list-specific membership, ordering
and editing behavior rather than forcing every view into an identical layout.

Use Ant Design components and tokens for generic controls and states. Reuse the
shared reference renderer and existing product primitives where applicable. Give
new shared components a concrete responsibility and existing consumers; avoid
feature-local copies or a speculative universal layout framework.

Review the proposed interaction and visual changes with the user before code
changes. Apply the installed UI design, Ant Design and interface-review skills.
Have Astra agents review real interactions and screenshots in light/dark and
desktop/phone layouts, using Safari where available or the approved Playwright
fallback. Verify list editing and record navigation alongside visual consistency.

## Sequence and completion

This is a separate follow-up to the full-corpus search evaluation, not part of
that evaluation's implementation scope. Completion requires a cross-surface
comparison, justified component reuse, behavior checks and visual/user review.
A saved-list-only cosmetic pass does not establish application-wide consistency.

## Reported reading-width inconsistency

Create Demiplane shows some content constrained to roughly half the available
reading space while horizontal rules span the full space. Review the shared
document layout and inherited styles for paragraphs, lists, other blocks and
horizontal rules. Agree a coherent reading width with the user before changing
it; narrower prose can be deliberate, but mixed block widths should be intentional.
Check wide and narrow screens and the shared full-page/preview presentations.

## References

- [Architecture overview](../../../architecture/overview.md)
- [Frontend guidelines](../../../../web/atlas-ui/docs/frontend-guidelines.md)
- [Web filter UX](../../items/rust-web-filter-ux-expansion.md)
- [Record detail polish](../../items/rust-web-record-detail-polish.md)
