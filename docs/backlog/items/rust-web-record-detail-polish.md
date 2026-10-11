# Rust Web Record Detail Polish

Status: in_progress
Priority: soon
Owner: unassigned
Last reviewed: 2026-10-10

## Problem

The web UI renders transient semantic family facts and prepared HTML through
shared presentation components. Creature, hazard and authored spell reference
have deliberate layouts; other families retain content and narrow physical/table
facts while their dedicated presentation remains future work.

Further detail work should improve source-backed semantic contracts only when a
consumer needs another fact, and reuse shared family presentation across detail,
preview, comparison and encounters.

## Current implementation scope

The current repair reuses shared record insets, wrapping identity/traits and
measured prose across detail and compact previews. Shared detail recovery retains
valid content during refresh of the same complete address, keeps failures local,
and offers retry and back navigation. One closed secondary disclosure owns lazy
references, remaster, variant and similar tabs; incoming references navigate the
actual referring owner and field, while comparison reuses the same family renderer.

Authored action costs use shared accessible glyphs in typed facts and prepared
content. Dedicated
equipment, journal and table presentation remains future family work, rather than
an expansion of this shared workflow slice.

Reference layouts put prose first and group facts/mechanics by actual pane width.
Owned abilities load their exact prepared detail inline on expansion, retaining
parent context and native full-page navigation. Casting entries show checked
parent membership and local spell ranks without executing selected-rank changes.
Hazards group Detection content and use explicit backend defense-block presence;
valid zero saves and unavailable values remain visible. Known-empty optional
content has no heading. Rank preview has a separate bounded-design backlog item.

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

- Extend narrow app-service family facts when current consumer data is insufficient;
  do not reintroduce RichDocument or a generic section/value/profile contract.
- Do not duplicate rich document parsing or record-kind semantics in TypeScript.
- Keep browser-specific layout code separate from shared Rust presentation contracts.
- Reuse family components and shared Ant primitives rather than duplicating
  layouts or creating visually inconsistent feature-local designs.

## Related

- [Architecture overview](../../architecture/overview.md)
- [ADR 0029: Local web app boundary](../../architecture/decisions/0029-local-web-app-boundary.md)
