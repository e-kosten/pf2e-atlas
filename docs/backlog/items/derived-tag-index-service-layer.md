# Derived-Tag Index Service Layer

Status: proposed  
Priority: soon  
Owner: unassigned  
Last reviewed: 2026-04-26

## Problem

The repo currently has a clear long-lived product-facing backend in `Pf2eDataService`, but the derived-tag editorial, discovery, evaluation, and app-side ontology cache paths still rely on a much weaker storage boundary.

That boundary is mostly:

- `src/app/storage-service.ts` opening `DatabaseSync`
- TUI/editorial composition passing `openIndex()` down into workbench flows
- tag discovery, evaluation, and editorial modules accepting raw `DatabaseSync`
- several CLI entrypoints constructing `DatabaseSync` directly

This creates an escape hatch instead of a real service layer. Connection ownership stays nominally centralized, but query policy, schema-tolerant record loading, cache writes, and index-backed derived-tag workflows are still spread across many modules.

The result is a backend split with one mature side and one under-abstracted side:

- `Pf2eDataService` is long-lived, typed, and capability-oriented
- the derived-tag side is still largely "open SQLite and let feature code operate on it"

## Desired Outcome

Introduce a real long-lived shared service layer for the derived-tag index-backed application surface without forcing this work back through `Pf2eDataService`.

The new architecture should:

- preserve the distinction between product-facing lookup/search/rule APIs and derived-tag editorial/discovery/evaluation workflows
- replace raw `openIndex()` and raw `DatabaseSync` seams with explicit typed derived-tag index services
- make index lifecycle, read/write policy, and comparison-runtime setup explicit at the service boundary
- give editorial UI, discovery tooling, evaluation tooling, and ontology cache workflows one shared backend instead of many direct SQL entrypoints
- remove the current raw-handle escape hatch once the new service layer is in place

## Constraints

- Do not collapse derived-tag editorial/discovery/evaluation work into `Pf2eDataService` just because both sides read the same prepared index.
- Do not keep `openIndex()` as a parallel long-term escape hatch once the new service layer exists.
- Keep authored-file mutation in `src/tags/editorial/writeback/`; the new service layer owns index-backed workflows, not source-file writeback.
- Keep app-owned cache writes explicit. The derived-tag ontology explorer cache currently writes into the SQLite index and needs a deliberate owner.
- Preserve the distinction between single-index workflows and dual-index comparison workflows such as movement evaluation.
- Finish the refactor as an end-state replacement. Do not leave mixed "some callers use the new service, some still take raw `DatabaseSync`" as the steady state.
- Update architecture docs and lint enforcement in the same task so the intended replacement path becomes the documented and enforced one.

## Notes

### Recommended service shape

The replacement should be a derived-tag-specific backend family, not one giant service class.

The intended shape is:

- one long-lived `DerivedTagIndexRuntime` or equivalent owner for index lifecycle, mode, and shutdown
- one or more focused facades built on top of that runtime, such as:
  - `DerivedTagEditorialDataService`
  - `DerivedTagDiscoveryDataService`
  - `DerivedTagEvaluationService`
  - `DerivedTagOntologyCacheService`
  - `DerivedTagComparisonService` for baseline/current movement workflows

Exact names can change, but the important architectural point is:

- callers receive typed derived-tag capabilities
- callers do not receive raw `DatabaseSync`

### Capabilities the new service layer needs to encode

The service family needs to cover the full capability set that is currently hidden behind direct DB access.

1. Index runtime and lifecycle

- open the prepared index once for long-lived derived-tag workflows
- expose explicit read-only vs read-write modes
- expose explicit close/shutdown ownership
- support comparison runtimes for baseline/current movement evaluation
- hide schema checks and optional-table probing that are currently done at feature level

2. Editorial session data

- count and load migration-session record slices by category, subcategory, record keys, required tag, untagged-only flag, and limit
- load enriched entity records plus reference edges for review sessions
- provide category scope summaries needed by workbench prompts and session creation
- support workbench session creation flows without passing raw DB handles into UI-level prompt helpers

3. Derived-tag ontology and explorer cache support

- load the derived-tag ontology explorer data snapshot
- read and write the explorer cache stored in the index
- expose a stable cache/revision key for UI and refresh workflows
- own the app-side derived-tag ontology cache policy instead of leaving it as a public DB helper

4. Discovery corpus access

- load filtered discovery records with optional vectors, derived tags, and references
- resolve exemplars by record key, canonical name, or alias
- expose a shared typed discovery record contract to discovery and evaluation clients

5. Discovery workflows

- semantic candidate discovery
- ruleable cohort discovery
- untagged cohort discovery

6. Evaluation workflows

- evidence analysis
- gap evaluation
- movement evaluation over two indexes

7. Policy and typing

- return typed editorial/discovery/evaluation models instead of SQL rows
- hide prepared statements, fallback projections, optional joins, embedding joins, and cache-table details
- make the write policy explicit so cache writes are allowed where intended, but ad hoc feature-level storage access is not

### Rewrite targets

The following surfaces currently need direct DB access or direct index-opening seams and should be rewritten to consume the new service layer.

App/runtime and composition seams:

- `src/app/storage-service.ts`
- `src/tui/app-services.ts`
- `src/refresh-index.ts`

App-side derived-tag ontology cache and explorer storage:

- `src/app/ontology/derived-tag-explorer-storage.ts`

Editorial workbench and session flows:

- `src/tags/editorial.ts`
- `src/tags/editorial/ui/workbench-controller.ts`
- `src/tags/editorial/ui/workbench-session-prompts.ts`
- `src/tags/editorial/sessions/session-builder.ts`
- `src/tags/editorial/sessions/record-loader.ts`
- `src/tags/editorial/sessions/category-scope-summary.ts`
- `src/tags/cli/editorial/create-derived-tag-migration-session.ts`

Discovery shared data/workflow modules:

- `src/tags/discovery/discovery-records.ts`
- `src/tags/discovery/semantic-discovery.ts`
- `src/tags/discovery/cohort-discovery.ts`
- `src/tags/discovery/untagged-cohort-discovery.ts`

Evaluation shared workflow modules:

- `src/tags/evaluation/evidence-analyzer.ts`
- `src/tags/evaluation/gap-evaluator.ts`
- `src/tags/evaluation/movement-evaluator.ts`

Discovery CLI entrypoints that currently open the index directly:

- `src/tags/cli/discovery/discover-semantic-candidates.ts`
- `src/tags/cli/discovery/discover-ruleable-cohorts.ts`
- `src/tags/cli/discovery/discover-untagged-cohorts.ts`
- `src/tags/cli/discovery/cluster-derived-tag-candidates.ts`

Evaluation CLI entrypoints that currently open the index directly:

- `src/tags/cli/evaluation/analyze-derived-tag-evidence.ts`
- `src/tags/cli/evaluation/evaluate-gaps.ts`
- `src/tags/cli/evaluation/evaluate-movement.ts`

Documentation and boundary-enforcement follow-through:

- `docs/architecture/overview.md`
- `docs/architecture/boundaries.md`
- `docs/architecture/editorial.md`
- relevant ADRs under `docs/architecture/decisions/`
- `eslint.config.js`
- `eslint-local-rules.js`

### Legacy surfaces to remove or retire

Once the new service layer is complete, the following legacy seams should not remain as general-purpose pathways.

Raw app storage seam:

- `Pf2eApplicationIndexHandle`
- `Pf2eApplicationStorageService`
- `openPf2eApplicationIndex(...)`
- `openConfiguredPf2eApplicationIndex(...)`
- `createPf2eApplicationStorageService(...)`

Workbench-specific raw-open seam:

- top-level editorial facade exports that currently expose the raw-open workbench contract
- `DerivedTagWorkbenchServices.openIndex`
- `DerivedTagWorkbenchOntologyHandle`
- `openDerivedTagWorkbenchOntology(...)`

Public helper contracts that take raw `DatabaseSync` for derived-tag workflows:

- editorial session/data helpers that currently accept `db: DatabaseSync`
- discovery helpers that currently accept `db: DatabaseSync`
- evaluation helpers that currently accept `db: DatabaseSync`
- app-side derived-tag ontology cache helpers that currently accept `db: DatabaseSync`

Direct CLI-owned `new DatabaseSync(...)` openings for derived-tag discovery/evaluation/editorial workflows:

- these should move to explicit runtime/service creation entrypoints instead of remaining command-local connection setup

The target end state is that raw `DatabaseSync` remains allowed only in true storage owners such as the derived-tag index runtime implementation itself, approved refresh/build ownership points, and any intentionally retained low-level comparison/runtime loaders.

### Suggested implementation slices

1. Introduce the derived-tag runtime/service owners and move shared index lifecycle there.
2. Migrate editorial session and workbench flows off `openIndex()` and raw DB parameters.
3. Migrate discovery shared data loaders and discovery workflows.
4. Migrate evaluation workflows, including the explicit comparison-runtime path for movement evaluation.
5. Move derived-tag ontology explorer cache loading/writing behind the new service family.
6. Rewrite CLI entrypoints to construct service runtimes instead of `DatabaseSync`.
7. Remove retired storage seams and add lint enforcement that prevents reintroduction.
8. Update architecture docs and ADRs to describe the new backend split.

### Validation focus

Future implementation work should explicitly validate:

- editorial workbench session creation still works in every mode
- derived-tag ontology explorer cache build/load behavior still works
- discovery commands still produce equivalent result shapes
- evidence/gap/movement evaluation commands still produce equivalent result shapes
- no editorial/discovery/evaluation caller outside the new runtime/service owners still imports `node:sqlite` or accepts raw `DatabaseSync`
- the repo’s architecture docs describe the same backend split the code now implements

## Related

- [Architecture overview](../../architecture/overview.md)
- [Architectural boundaries](../../architecture/boundaries.md)
- [Editorial architecture](../../architecture/editorial.md)
- [Tagging tooling reorganization](./tagging-tooling-reorg.md)
- [Derived-tag concept model implementation](./derived-tag-concept-model-implementation.md)
