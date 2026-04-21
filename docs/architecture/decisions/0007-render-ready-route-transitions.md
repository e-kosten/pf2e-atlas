# ADR 0007: Render-Ready Route Transitions

- Status: Accepted
- Date: 2026-04-20

## Context

The TUI adopted a shared navigation coordinator, but route readiness was still partly conventional:

- some routes were prepared before commit and mounted with ready payloads
- other routes committed first and then loaded the data needed for their first meaningful frame after mount
- ontology/query launch behavior still carried launch-mode shims instead of explicit navigation intent

That left room for visible hangs, flicker, and architectural drift. The ontology area was the clearest example: the route changed first, then `loadSearchSemanticsDomain()` ran from app render logic, so the shared transition loader could not represent the work the user was waiting on.

## Decision

Treat render readiness as a navigation-owned contract instead of a screen-local convention.

In practice this means:

- every user-visible route change goes through the navigation subsystem
- routes whose first meaningful frame depends on loaded data must be prepared before commit
- route screens mount from render-ready payloads instead of running route-entry bootstrap work after navigation
- ontology/query launch behavior uses explicit navigation intents rather than loose launch-mode flags
- the currently mounted screen shows one shared transition-status treatment while navigation prepares the next route

## Consequences

- The ontology area opens only after the search-semantics model is ready, so the current screen can show the shared transition footer instead of exposing a post-navigation hang.
- Search result-reader entry mounts from a prepared session and should not auto-execute a navigation-origin query on mount.
- Route creation is centralized behind navigation-owned helpers, so app-shell callbacks request intent rather than constructing partial route payloads directly.
- Lint, docs, and tests should enforce this boundary once the migration is complete so route-entry loading paths do not regrow as convention-only exceptions.
