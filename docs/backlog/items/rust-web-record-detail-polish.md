# Rust Web Record Detail Polish

Status: in_progress
Priority: soon
Owner: unassigned
Last reviewed: 2026-10-13

## Problem

The web UI renders transient semantic family facts and prepared HTML through
shared presentation components. Creature, hazard and authored spell reference
have deliberate layouts; other families retain content and narrow physical/table
facts while their dedicated presentation remains future work.

Further detail work should improve source-backed semantic contracts only when a
consumer needs another fact, and reuse shared family presentation across detail,
preview, comparison and encounters.

## Current Baseline

Reference, preview, comparison and encounter surfaces reuse semantic family
components and selected prepared HTML. Exact owner addresses govern navigation,
Related availability and bounded owned reading. Similar discovery uses the shared
search browser with explicit seed/filter state. These contracts are documented in
[ADR 0047](../../architecture/decisions/0047-semantic-record-presentation.md) and
[frontend guidelines](../../../web/atlas-ui/docs/frontend-guidelines.md).

Creature, hazard and authored spell reference have dedicated layouts. Equipment,
journal and table presentation and selected-rank execution remain separate design
work. Further changes should preserve useful content, availability states,
source attribution and consistent behavior across hosts.

## Desired Outcome

Improve the web record detail view for scanability, navigation, and PF2e-specific readability.

The pass should consider:

- stronger layout for identity facts, badges, sections, and rich content;
- dedicated equipment, journal and table layouts where useful;
- inline reference navigation and first-class record-detail routes;
- loading, empty, error, and stale-detail states;
- detail-pane behavior when results change, panes collapse, or users navigate by keyboard;
- visual polish that stays consistent with the selected Ant Design-based UI.
- meaningful casting-entry context for owned spells, resolved from their actual
  source owner through the existing service rather than displaying a raw local
  Foundry identifier or resolving it in the browser.
- [bounded spell rank preview](rust-spell-rank-preview.md), designed separately
  from reference layout and authored heightening display.

## Constraints

- Revisit creature spell rosters' Base rank / Cast rank presentation as a separate
  product design discussion. It remains unchanged by the reference workspace pass.

- Extend narrow app-service family facts when current consumer data is insufficient;
  keep checked source and prepared HTML authoritative under ADR 0047.
- Do not duplicate rich document parsing or record-kind semantics in TypeScript.
- Keep browser-specific layout code separate from shared Rust presentation contracts.
- Reuse family components and shared Ant primitives rather than duplicating
  layouts or creating visually inconsistent feature-local designs.

## Related

- [Architecture overview](../../architecture/overview.md)
- [ADR 0029: Local web app boundary](../../architecture/decisions/0029-local-web-app-boundary.md)
