# ADR 0015: Shared Explorer Host Contract And Live Group Editing

- Status: Accepted
- Date: 2026-04-26

## Context

The TUI already has a largely shared explorer interaction layer in `src/tui/filter-explorer/`, but the host contracts above that layer are still uneven:

- ontology browse is route-backed and leaf-driven toward seeded search or results
- search editing is session-backed and still carries grouped-editing behavior that is partly staged and partly host-local
- discovery-mode switching is exposed through one shared affordance, but the host/model meaning of those modes already differs between ontology and search
- explorer-backed search edit flows still treat root exit too much like apply-and-close instead of a nested local editing step

At the same time, the grouped search-editing product direction has become clearer:

- users should be able to stay inside one structural group and refine it live
- counts and option narrowing should react to those in-progress edits immediately
- returning from a nested field flow should normally preserve the edits already made
- broad staged group-draft editing adds friction that works against that live feel

The architecture therefore needs a clearer split between:

- the shared explorer interaction family
- the host-specific semantics layered above it
- the search-specific grouped-editing behavior that mutates the live query tree

## Decision

Treat `src/tui/filter-explorer/` as the shared explorer interaction owner, and build future grouped search editing on top of a stronger shared explorer-host contract rather than creating a second interaction family.

In practice this means:

- the shared explorer layer should own explorer interaction outcomes and shared host glue where that glue is genuinely generic
- the shared explorer outcome vocabulary is `back`, `exitRoot`, `cancel`, and `selectTarget`
- target activation should stay generic in the shared layer, with host-provided activation styles such as `open`, `toggle`, `edit`, and `none`
- host-provided selection-state presentation may augment inspect-and-open sessions when a host needs live row badges, focused-clause summaries, or similar editor affordances without promoting those semantics into a shared canonical explorer mode
- ontology and search hosts should continue to own their own route/session semantics, model loading, and leaf behavior
- discovery-mode affordances may be shared, but ontology and search should own their own mode types and the actual meaning of those modes
- grouped search editing should move toward live in-place mutation of the focused structural block in the canonical query tree
- structured search editing owns a host-level continuation coordinator above prompt and explorer child flows; the coordinator interprets shared explorer outcomes into search-owned resume and mutation semantics
- group-local structured-editor continuation should use canonical `groupPath` as the normal resume anchor, while exact node paths remain reserved for genuinely node-scoped operations
- shared-explorer-backed leaf edits and grouped field edits should converge on the same host-owned continuation path while preserving distinct bounded mutation kinds
- structured search editing classifies field-edit intent before opening child flows; the durable semantic edit routes are grouped-field cohort editing and single-clause leaf editing
- field-edit classification uses semantic route intents and a shared route catalog; TUI picker rows are display/session data and do not select route behavior
- scope is a root-singleton leaf route, and query-state normalization prunes scope-dependent metadata, metric, and action-cost clauses when the category changes
- metric key discovery is an ordinary leaf child surface rather than a fallback edit route
- `linksTo` and `linkedFrom` remain canonical record-link leaves and are edited as executable leaf routes when surfaced by the structured editor
- generic draft/query helpers below the explorer seam may emit generic replace or insert outputs for prompt-local or select-target child surfaces, but structured-editor shared-explorer writeback requires an explicit host mutation builder instead of treating generic serialization as the default
- a dedicated search-specific tree editor may remain above the shared explorer layer when it acts as a live host surface over canonical query state rather than reviving a separate staged query model
- broad group-level staged draft editing should not be the primary model for search-group refinement
- prompt-local drafts remain appropriate for incomplete value-entry flows such as numeric scalar input, but those small drafts should not expand into a second long-lived query model
- editor-only identity or session baggage should not be written into the canonical `SearchRequest` / `SearchFilterNode` contract

## Consequences

- ontology browse and search editing continue to feel like one explorer interaction family even though their host behaviors remain distinct
- the shared explorer surface can support both ontology browse rows and search live-selection rows through host-provided presentation and activation-style adapters instead of subclasses or host-specific rendering branches
- search-group editing can update counts and option availability from the real current query as the user edits a structural block
- `Esc` may return the user from a nested field flow to the surrounding tree view while preserving already-applied live edits
- search may keep a separate tree-editor surface, but that surface must remain a live host over canonical search state rather than a parallel staged editing architecture
- search-specific grouped editing keeps its own tree/block mutation logic instead of pushing that state into the shared explorer layer
- future structured-editor child flows should add tests under the named continuation, state, helper, or structured-editor interaction surfaces rather than only extending broad search-screen smoke coverage
- future divergence in ontology versus search mode vocabularies will not require undoing the shared explorer contract, because the shared layer does not own one canonical mode type
- implementation work in this area should remove duplicated host glue where it is truly generic, but should not collapse ontology leaf behavior, search leaf behavior, and search-group mutation semantics into one universal host model
