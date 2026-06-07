# Rust Web Record Detail Polish

Status: proposed
Priority: soon
Owner: unassigned
Last reviewed: 2026-06-07

## Problem

The web UI can load and render record detail through the shared presentation document, but the detail experience is still a first vertical slice. It needs a readability and interaction pass before users will rely on it for browsing records at length.

Record details should benefit all consumers by improving shared presentation contracts when the base document is insufficient, rather than inventing browser-only record data models.

## Desired Outcome

Improve the web record detail view for scanability, navigation, and PF2e-specific readability.

The pass should consider:

- stronger layout for identity facts, badges, sections, and rich content;
- kind-specific presentation improvements where the shared `RecordPresentationDocument` supports them;
- inline reference navigation and first-class record-detail routes;
- loading, empty, error, and stale-detail states;
- detail-pane behavior when results change, panes collapse, or users navigate by keyboard;
- visual polish that stays consistent with the selected Ant Design-based UI.

## Constraints

- Extend or improve `RecordPresentationDocument` when shared presentation data is insufficient.
- Do not duplicate rich document parsing or record-kind semantics in TypeScript.
- Keep browser-specific layout code separate from shared Rust presentation contracts.

## Related

- [Architecture overview](../../architecture/overview.md)
- [ADR 0029: Local web app boundary](../../architecture/decisions/0029-local-web-app-boundary.md)
