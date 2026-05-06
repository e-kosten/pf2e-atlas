# Search Semantics Surface Convergence

Status: proposed  
Priority: soon  
Owner: unassigned  
Last reviewed: 2026-04-26

## Problem

The repo currently has two related but distinct search-semantics surfaces:

- a broad ontology browser loaded through `loadSearchSemanticsDomain()` and built by `buildSearchSemanticsDomain(...)`
- a prepared search-scoped explorer loaded through `loadSearchFilterExplorerDomain({ request, discoveryMode, targetFields })` and built by `buildPreparedSearchFilterExplorerDomain(...)`

These surfaces already share important lower-level plumbing:

- the `OntologyDomainModel` / `OntologyNode` tree contract
- the shared filter-explorer list/detail shell and action rail
- some builder helpers and derived-tag assembly logic under `src/app/ontology/search-semantics-domain.ts`

But they still diverge in ways that are only partly intentional:

- the ontology browser is catalog/browse oriented and can exist without a live `SearchRequest`
- the prepared explorer is query/applicability oriented and requires a concrete scoped `SearchRequest`
- similar tree sections are assembled through separate top-level builder paths with partially duplicated logic
- the product meaning of `matching` vs `catalog` is real in the prepared explorer but only partly expressed in the ontology browser

That leaves the repo with a clear architectural opportunity:

- preserve the legitimate product distinction between broad browse and prepared query-scoped exploration
- while converging more of the internal tree-assembly primitives, terminology, and mode semantics where they should actually match

This item tracks that larger opportunity and the open decisions around it, rather than treating each symptom as an isolated bug.

## Desired Outcome

Clarify and incrementally improve the shared architecture for search-semantics surfaces so the repo has:

1. two explicit public surface entrypoints with distinct jobs
2. a more converged internal assembly model where duplication is accidental rather than intentional
3. a clearer product contract for `matching` vs `catalog`, especially for derived tags
4. documented guidance for what should stay shared across ontology/search/picker flows and what should remain surface-specific

The expected end state is not one universal builder that erases the difference between the surfaces. The intended direction is:

- keep a broad ontology browser that can exist without a live query
- keep a prepared search-scoped explorer that is shaped by a concrete canonical `SearchRequest`
- reduce accidental divergence in builder internals, count semantics, and action affordances

## Current Architecture Snapshot

### Broad ontology browser

- public loader: `loadSearchSemanticsDomain()`
- public builder: `buildSearchSemanticsDomain(...)`
- primary job: browse the search ontology and category/field/tag catalog before or apart from a specific query

### Prepared search-scoped explorer

- public loader: `loadSearchFilterExplorerDomain({ request, discoveryMode, targetFields })`
- public builder: `buildPreparedSearchFilterExplorerDomain(...)`
- primary job: show values, counts, and field/tag applicability for one concrete scoped query
- search-hosted single-field flows pass `targetFields` so prepared discovery does not load unrelated fields or metric families

### Shared lower-level seams

- `OntologyDomainModel` / `OntologyNode`
- shared filter-explorer controller and list/detail shell
- shared action rail
- shared search-semantics domain module

### Surface-specific seams

- ontology route/workflow orchestration
- search-draft/apply orchestration
- picker-local selection flows
- prepared reader setup from `SearchRequest` and `discoveryMode`

## Open Product And Architecture Decisions

### 1. Matching vs catalog in the ontology browser

The ontology browser now exposes discovery-mode switching through the action rail, but the deeper ontology model semantics are still unresolved.

Open questions:

- Should ontology use the same core `matching` / `catalog` concept as the prepared search explorer?
- Or should ontology use different user-facing wording even if the underlying concept remains the same?
- Is the practical ontology distinction mainly “show only nonzero values” vs “show the full catalog including zero-count values”?

### 2. Derived-tag family counts and child visibility

Current derived-tag family behavior mixes semantics:

- family rows can show catalog-sized counts such as `Function | 14 tags`
- drilling into a family can still show only the subset with live counts

Open questions:

- In matching mode, should a family count mean “number of tags with nonzero matches”?
- In catalog mode, should a family count mean “full family size”?
- Should catalog mode always show zero-count tags explicitly?

### 3. Search explorer vs ontology browser alignment

The prepared search explorer already has real mode-aware behavior, while the ontology browser is still partly a wiring-first shell over broader browse data.

Open questions:

- Which semantics should be identical across both surfaces?
- Which differences are legitimate because one surface is browse-first and the other is query-first?
- How much of the current duplicate tree assembly can be factored into shared internal helpers without obscuring the two distinct public surface entrypoints?

### 4. Explorer vs picker behavior

The repo currently has both:

- explorer-level discovery-mode switching through the action rail
- picker-local discovery-mode switching inside modal selection flows

Open questions:

- Which flows should standardize on explorer-level action-rail switching?
- Which flows legitimately remain picker-local?
- How much terminology and description logic should be shared across those two interaction families?

## Open Implementation Follow-Through

This item is also the right umbrella for several still-open issues that cluster around the same architecture seam:

- the search-side derived-tag surface can still appear empty for broad scoped cases such as only `scope=creature`
- ontology derived-tag matching/catalog semantics are still mixed rather than explicit
- ontology family-node action affordances and discovery behavior still need follow-through now that the basic wiring exists
- the current builder split in `src/app/ontology/search-semantics-domain.ts` likely contains refactorable duplication that should be cataloged before future feature work deepens it

These should not all be treated as one commit or one immediate refactor, but they belong to the same tracked architectural area.

## Constraints

- Preserve separate public surface entrypoints for broad browse and prepared query-scoped exploration unless an explicit architectural decision says otherwise.
- Do not collapse ontology browser, prepared explorer, and picker flows into one generic builder or one generic workflow owner.
- Prefer convergence of internal assembly helpers, shared terminology, and shared semantics over compatibility layers or duplicated parallel fixes.
- Keep `ontology-service` as the public facade layer and preserve documented storage/data boundaries.
- Any deeper semantic change for `matching` / `catalog` should be reflected in the architecture docs and the relevant runnable screen or app tests.

## Suggested Next Steps

1. Catalog the overlapping helper families inside `src/app/ontology/search-semantics-domain.ts` and identify which ones can be shared without hiding the surface distinction.
2. Decide the intended ontology meaning of `matching` vs `catalog`, especially for derived-tag families and zero-count tag visibility.
3. Fix the search-side empty derived-tag path for broad scoped requests so the prepared explorer remains a trustworthy semantic reference.
4. Align ontology family counts, child lists, and action affordances to the chosen semantics.
5. Update `docs/architecture/tui.md` and any relevant ADR if the internal builder split or discovery-mode contract changes materially.

## Related

- [TUI architecture](../../architecture/tui.md)
- [Architectural boundaries](../../architecture/boundaries.md)
- [Search Semantics Explorer Completeness](./search-semantics-explorer-completeness.md)
- [Shared TUI interaction family contracts](./shared-tui-interaction-family-contracts.md)
- [ADR 0002: Readonly ontology and explicit storage boundary](../../architecture/decisions/0002-readonly-ontology-and-explicit-storage-boundary.md)
- [ADR 0005: Live search-semantics exploration](../../architecture/decisions/0005-live-search-semantics-exploration.md)
- [ADR 0006: Shared TUI interaction contracts](../../architecture/decisions/0006-shared-tui-interaction-contracts.md)
