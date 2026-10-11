# Rust Web Record Detail Polish

Status: proposed
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

## Desired Outcome

Improve the web record detail view for scanability, navigation, and PF2e-specific readability.

The pass should consider:

- stronger layout for identity facts, badges, sections, and rich content;
- dedicated equipment, journal and table layouts where useful;
- inline reference navigation and first-class record-detail routes;
- loading, empty, error, and stale-detail states;
- detail-pane behavior when results change, panes collapse, or users navigate by keyboard;
- visual polish that stays consistent with the selected Ant Design-based UI.

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
