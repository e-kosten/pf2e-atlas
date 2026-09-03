# ADR 0030: Local State Database

Status: accepted  
Date: 2026-06-16

## Context

PF2e Atlas writes a generated SQLite artifact from the Foundry PF2E source. Runtime retrieval opens that artifact read-only, and rebuilds publish a replacement artifact after writing a complete temporary database. That model keeps source-derived search, lookup, graph, filter, FTS, and embedding data deterministic and rebuildable.

Saved lists introduce durable mutable state. Users and local agents need to create and edit lists over time, and those lists must survive artifact rebuilds. Storing that state inside the generated artifact would require rebuild-time preservation, user-table copying, write coordination with web/CLI processes, and validation rules that distinguish generated rows from mutable local rows.

## Decision

PF2e Atlas stores durable mutable local state in a separate local-state SQLite database, resolved beside the active generated artifact.

The initial database file is:

```text
pf2e-local-state.sqlite
```

The generated artifact remains the owner of source-derived runtime data and is still rebuildable as a replaceable artifact. The local-state database owns user-authored or agent-authored durable local state such as saved lists.

The implementation uses an `atlas-local-state` crate. It owns:

- local-state schema metadata and migrations
- saved-list storage tables and product APIs
- stable generated saved-list keys, CLI-friendly saved-list slugs, and list-ref resolution by key or slug
- saved-list metadata tags for user-authored grouping and product-surface filtering
- durable item ordering
- saved item snapshots used when an active artifact no longer contains a saved record key
- saved-list item insertion from already-resolved canonical record inputs
- encounter participant state, including typed spell-resource initialization baselines and current remaining values

Participant spell resources use a closed storage key matching the app contract: prepared slot, spontaneous rank pool, innate occurrence use, or focus pool. Stateless at-will spells do not create resource rows. App-service derives these keys from typed canonical occurrence ownership and asks local state to initialize supported current counts once; local state does not parse record labels or source prose and does not implement daily-rest or refocus rules.

Schema v7 captures an immutable mechanical baseline in the same transaction that creates each new encounter participant. The generic participant reset transaction restores HP/max/temp, defeated state, conditions, initiative and current-turn participation, variant/adjustment state, derived action-budget state, and every registered spell-resource row; it preserves display name, notes, visibility, and side. The reset result enumerates its registered domains, and the exhaustive app-service mapping makes a newly registered local-state domain a compile-time integration obligation. Migration creates the baseline table without backfilling: a legacy participant without a genuine creation baseline exposes reset unavailable until it is explicitly recreated. Local state never treats migration-time current state or later canonical defaults as the original baseline.

`atlas-runtime` resolves the local-state path beside the active index path. Product surfaces compose through runtime path resolution and local-state product APIs; they do not store mutable user state in `atlas-index`. Cross-layer saved-list workflows that need active artifact context, such as resolving a record name before adding it to a list or hydrating saved rows for display, belong in `atlas-app-service`.

## Consequences

Artifact rebuilds do not need to preserve saved-list rows because they never own those rows.

Saved-list items store canonical record keys plus display snapshots. Adding an item requires strict resolution to a single record key before local-state insertion, but the resolution workflow is owned by app-service/search composition rather than by local-state storage. Later artifact rebuilds may make a key unresolved. Unresolved items are preserved and surfaced explicitly instead of being silently deleted.

Import/export for local state remains a follow-up product feature. It should serialize local-state entities through stable, scriptable formats rather than making users copy generated artifacts or mutable SQLite files by hand.

## Boundaries

- `atlas-index` owns generated artifact schema, validation, read APIs, and artifact writing.
- `atlas-local-state` owns mutable local-state schema plus product APIs for interacting with saved lists and future durable local-state domains.
- `atlas-runtime` owns path resolution for both artifact and local-state paths.
- `atlas-app-service` owns workflows that compose active artifact retrieval with local-state product APIs.
- `atlas-cli` owns command grammar, output, and exit codes for saved-list commands and calls app-service workflows through its client facade.
- Future web/TUI saved-list UI should route through app-service/local-state APIs, not direct frontend SQLite access.
