# Derived-Tag Editorial Architecture

This document describes the current architecture of the derived-tag subsystem under `src/tags/` after the Wave 2 split. It explains which folders now own runtime publication, durable review inputs, editorial session state, writeback, review UI, and the grouped CLI layout.

For broader project layering, read [overview.md](./overview.md) and [boundaries.md](./boundaries.md) first. For the non-obvious decisions this document relies on, see:

- [0001 Shared Backend, Separate MCP and Editorial Surfaces](./decisions/0001-shared-backend-separate-surfaces.md)
- [0002 Readonly Ontology and Explicit Storage Boundaries](./decisions/0002-readonly-ontology-and-explicit-storage-boundary.md)
- [0003 Lint-Enforced Derived-Tag Boundaries](./decisions/0003-lint-enforced-derived-tag-boundaries.md)

## Purpose

`src/tags/` exists to support a human-in-the-loop editorial workflow around derived tags:

- define the authored ontology and explainable rules
- carry explicit assignments, exemplars, and durable review registries
- inspect the live indexed corpus for missing, over-broad, or repeatedly rejected coverage
- build editorial sessions from pending review queues, migration inputs, and evidence passes
- write approved changes back into authored TypeScript sources

This subsystem is partly runtime infrastructure and partly editorial tooling. The runtime portion publishes the canonical derived-tag model used by indexing and record derivation. The editorial portion clones that authored state, builds review sessions, lets humans resolve them, and writes the approved state back to disk.

When evaluating the long-term shape of `src/tags/`, treat `legacy-rules/` and `legacy-seed-migrations/` as transitional placeholders only. They preserve old rule coverage while that content is migrated into the authored model; they are not the desired steady-state owner directories.

## Subsystem Shape

```mermaid
flowchart LR
  authored["`ontology/`, `rules/`, `assignments/`, `exemplars/`
  authored derived-tag source"]
  reviews["`reviews/`
  assignment reviews, memory, exemplar reviews,
  reviewed discovery negatives"]
  runtimePub["`runtime/publication/`
  publish ontology, exemplars, legacy seed indexes"]
  runtimeDerive["`runtime/derivation/`
  explicit assignments and final derivation API"]
  runtimeMatch["`runtime/matcher/`
  rule matcher and normalization"]
  runtimeCompat["`runtime/compat/`
  compatibility helpers only"]
  discovery["`discovery/`
  candidate mining and cohort discovery"]
  evaluation["`evaluation/`
  evidence, gaps, movement"]
  editorialState["`editorial/state/`
  mutable authored working state"]
  editorialSessions["`editorial/sessions/`
  session construction, record loading, persistence"]
  editorialWriteback["`editorial/writeback/`
  lint and authored-file import"]
  editorialUi["`editorial/ui/`
  workbench and review controllers"]
  cli["`cli/discovery/`, `cli/evaluation/`,
  `cli/editorial/`, `cli/shared/`
  grouped offline entrypoints"]
  app["`src/app/` + `src/tui/app-services.ts`
  shared runtime, storage, workbench composition"]
  index["SQLite index + embeddings"]

  authored --> runtimePub
  authored --> runtimeDerive
  authored --> runtimeMatch
  reviews --> runtimeDerive
  reviews --> editorialState
  runtimeMatch --> runtimeDerive
  runtimeCompat --> runtimeDerive

  runtimeDerive --> discovery
  runtimeDerive --> evaluation
  runtimeDerive --> editorialSessions
  runtimePub --> editorialState
  runtimePub --> editorialSessions

  index --> discovery
  index --> evaluation
  index --> editorialSessions
  index --> app

  discovery --> editorialSessions
  evaluation --> editorialSessions
  editorialState --> editorialSessions
  editorialSessions --> editorialWriteback
  editorialSessions --> editorialUi
  editorialWriteback --> authored
  editorialWriteback --> reviews

  cli --> discovery
  cli --> evaluation
  cli --> editorialSessions
  cli --> editorialWriteback
  app --> editorialUi
```

The important split is:

- authored directories hold the durable source of truth
- `reviews/` holds durable review inputs, including reviewed discovery negatives
- `runtime/` publishes authored truth into a derivation-ready model
- `editorial/` owns mutable session state, review sessions, writeback, and review UI
- `cli/` provides grouped offline entrypoints over those services

## Major Subareas

| Area | Owns | Consumes | Produces |
| --- | --- | --- | --- |
| `ontology/`, `rules/`, `assignments/`, `exemplars/` | Authored ontology, authored rules, explicit overrides, curated teaching sets | Shared manifest and domain contracts | The durable inputs the runtime and writeback flows operate on |
| `reviews/` | Pending assignment reviews, prior assignment memory, pending exemplar reviews, reviewed discovery negatives | Current editorial policy and managed categories | Durable review registries used by session building, discovery filtering, and writeback |
| `runtime/publication/` | Published ontology catalogs, exemplar publication, legacy seed publication, source catalogs | Authored ontology, exemplars, seed definitions | Flattened runtime registries and source-aware publication helpers |
| `runtime/derivation/` | Explicit assignment index, final tag derivation API, runtime-level assignment views | Published ontology, matcher outputs, review registries, legacy inputs | `deriveRecordTags`, source-aware derivations, pending-assignment views |
| `runtime/matcher/` | Rule matching engine, text/reference matching, tag normalization | Authored and legacy rules | Rule-match outputs reusable by derivation |
| `runtime/compat/` | Legacy family alias compatibility only | Published runtime vocabulary | Compatibility helpers for old family names; not primary runtime ownership |
| `discovery/` | Cohort mining, semantic clustering, family-gap discovery, discovery record decoding | Live SQLite index, embeddings, runtime helpers, reviewed discovery registries | Candidate cohorts and discovery artifacts suggesting possible new rules or assignments |
| `evaluation/` | Evidence analysis, gap evaluation, movement comparison | Discovery artifacts, live index state, runtime exemplar and family lookups | Decision-support reports for editorial review |
| `editorial/state/` | Mutable cloned authored state and published working-state summaries | Authored imports, published runtime registries, review registries | Session-safe working state, queue summaries, workbench ontology snapshots |
| `editorial/sessions/` | Record loading, actionable scope summaries, session building, session persistence | Editorial state, runtime publication, live index records, review registries | Review sessions and persisted scratch artifacts |
| `editorial/writeback/` | Session linting and import into authored files | Editorial working state, approved sessions | On-disk updates to authored assignments, rules, exemplars, and review registries |
| `editorial/ui/` | Review screen model/state, review controller, workbench prompts and controller | Session services, writeback services, app-composed storage/runtime services | Interactive review and workbench flows |
| `cli/discovery/`, `cli/evaluation/`, `cli/editorial/`, `cli/shared/` | Grouped offline entrypoints plus shared scope parsing | Discovery, evaluation, editorial session/writeback services, config/index opening | Human-readable reports, session artifacts, and the workbench launcher |

## Runtime Ownership Split

The published derived-tag runtime is no longer one undifferentiated `runtime/` bucket.

### `runtime/publication/`

Owns publication of the authored model into flattened runtime registries:

- ontology publication and grouping
- exemplar publication
- catalog seed and legacy seed migration publication
- source-aware catalog helpers used by derivation and editorial tooling

### `runtime/derivation/`

Owns the runtime assembly that turns publication outputs into final derivation behavior:

- explicit assignment indexing
- final tag derivation APIs
- pending-assignment view helpers consumed by editorial state

### `runtime/matcher/`

Owns explainable rule matching:

- text, trait, family, and reference matching
- normalized matcher inputs
- reusable matcher-side tag normalization

### `runtime/compat/`

Owns compatibility helpers that keep older family names usable during migration. This is support code, not the primary owner for runtime semantics.

## Authored Truth And Review Registries

The durable editorial inputs now come from two different places:

- authored truth in `ontology/`, `rules/`, `assignments/`, and `exemplars/`
- review truth in `reviews/`

`reviews/` is not scratch space. It is durable authored input with its own ownership:

- `reviews/assignment-reviews/` holds pending include/exclude decisions
- `reviews/assignment-memory/` preserves prior reasoning and supporting context
- `reviews/exemplar-reviews/` holds pending keep/drop or polarity-review decisions
- `reviews/discovery-reviewed-records.ts` records reviewed-negative discovery outcomes so future family-gap passes can suppress or audit them intentionally

That reviewed-discovery registry used to be discoverable through `src/tags/discovery/discovery-reviewed-records.ts`; that file now exists as a compatibility re-export. The durable owner is `src/tags/reviews/discovery-reviewed-records.ts`.

## Discovery And Evaluation Loop

Discovery and evaluation still provide the evidence loop around the runtime, but they now consume the review registries more explicitly.

`discovery/` answers questions like:

- which untagged records cluster near the current exemplars?
- which cohorts look ruleable versus manual-only?
- which records were already reviewed negative for a family and should be excluded or audited by reason bucket?

`evaluation/` converts those signals into decision support:

- `evidence-analyzer.ts` explains common tokens, phrases, traits, and references
- `gap-evaluator.ts` measures likely missing coverage for an existing family or tag
- `movement-evaluator.ts` compares baseline and current indexes to catch regressions or weak gains

Those modules remain advisory. They surface evidence for editors, but they do not directly rewrite authored files.

## Editorial Execution Split

The editorial execution layer is now intentionally split by concern.

### `editorial/state/`

Owns mutable editorial working state:

- `authored-state.ts` clones the imported authored and review registries into a writable in-memory session state
- `working-runtime.ts` rebuilds the derivation-ready runtime view from the current in-memory authored state and caches it by authored-state revision
- `runtime-state.ts` republishes the current working state into queue summaries, pending-assignment views, and source-aware derivation views backed by that working runtime

This split keeps sessions from mutating imported modules in place while still letting the workbench operate against one coherent current snapshot.

### `editorial/sessions/`

Owns session construction and persistence:

- `session-builder.ts` builds review sessions from review queues, exemplar cleanup, proposal review, legacy seed migration, or legacy rule takeover worksets
- `record-loader.ts` loads the indexed record slices needed for those sessions
- `review-session.ts` owns review-item and progress calculations
- `session-store.ts` owns persisted scratch-session JSON artifacts
- scope helpers such as `actionable-session-scope.ts` and `category-scope-summary.ts` define what is actionable in each mode

### `editorial/writeback/`

Owns the only path that mutates authored files from session decisions:

- `authored-state-writer.ts` owns authored-file path selection, TypeScript serialization, and registry writeback
- `linter.ts` validates a session before import
- `importer.ts` applies approved session decisions back into authored assignments, rules, exemplars, and review registries

Writeback consumes the session output from `editorial/sessions/` and the mutable authored state from `editorial/state/`; it should remain the only owner of source-file mutation logic.

### `editorial/ui/`

Owns the interactive review/workbench surface:

- `workbench-controller.ts` opens indexes, builds sessions, and writes scratch artifacts for the TUI workbench
- `workbench-session-prompts.ts` owns the session-creation prompt-adapter seam and is the only editorial workbench module that should depend directly on the TUI prompt-adapter contract
- `review-controller.ts` coordinates persist/import actions by calling session-store and writeback services
- `review-ui-controller.ts`, `review-screen-model.ts`, `review-screen-state.ts`, and `review-ui.tsx` own the review screen behavior

`src/tui/app-services.ts` composes these services into the terminal app. The editorial UI is a surface over shared runtime, storage, and editorial services, not a second copy of editorial logic.

## Grouped CLI Layout

The CLI files under `src/tags/cli/` are grouped by the services they drive:

- `cli/discovery/` runs candidate-mining workflows such as ruleable cohorts, semantic candidates, and untagged-cohort discovery
- `cli/evaluation/` runs evidence, gap, and movement reports
- `cli/editorial/` creates sessions, reviews/imports sessions, summarizes review queues, and launches the migration workbench
- `cli/shared/` owns shared scope parsing through `search-scope-args.ts`

These entrypoints are intentionally thin. They should:

- parse scope arguments and validation flags
- open the configured SQLite index
- call one discovery, evaluation, session, or writeback service
- print a report or launch the TUI workbench

The special case is `cli/editorial/derived-tag-migration-workbench.ts`, which launches the terminal app rather than owning editorial logic itself. The actual workbench composition still happens through `src/tui/app-services.ts` and `editorial/ui/`.

## Stable Facades And Compatibility Entry Points

The stable non-tag facade remains `src/tags/index.ts`.

Inside the subsystem, the owning paths are the split directories themselves:

- `src/tags/runtime/{publication,derivation,matcher,compat}/`
- `src/tags/editorial/{state,sessions,writeback,ui}/`

Compatibility entrypoints should be introduced only when callers genuinely need a durable facade. They are not the default internal routing target.

## Boundaries To Preserve

The current codebase already encodes several intended boundaries:

- outside `src/tags`, callers should normally import derived-tag functionality through `src/tags/index.ts` or another approved facade instead of leaf tag modules
- reviewed discovery state belongs under `src/tags/reviews/`, not under discovery-owned scratch code
- ontology browsing is assembled through `src/app/ontology-service.ts` and treated as a readonly model, not mutable shared UI state
- direct SQLite opening is explicit through the application storage service, `src/tags/editorial/cli-utils.ts`, or grouped CLI entrypoints, not scattered through unrelated feature modules
- CLI scope parsing is centralized in `src/tags/cli/shared/search-scope-args.ts`
- discovery and evaluation can recommend changes, but `editorial/writeback/` owns writeback into authored files
- TUI workbench composition flows through `src/tui/app-services.ts`, not direct feature-level imports of editorial internals

Those rules are documented in `docs/architecture/boundaries.md` and reinforced by ESLint restrictions where the shared abstraction is already considered stable.

## End-To-End Editorial Flow

The steady-state maintenance loop now looks like this:

1. Editors maintain authored ontology, rules, assignments, exemplars, and review registries in `src/tags/`, with durable review inputs under `src/tags/reviews/`.
2. `runtime/publication/`, `runtime/matcher/`, and `runtime/derivation/` publish those inputs into the canonical derivation model used during indexing and derived-tag lookup.
3. Discovery and evaluation inspect the current SQLite index and embeddings, using review registries to suppress or audit already reviewed negatives.
4. `editorial/state/` clones the authored truth into a mutable working state and republishes queue summaries and workbench views from that current snapshot.
5. `editorial/sessions/` builds a review session from queued reviews, proposal-review work, legacy migration work, or evidence-driven cleanup.
6. The CLI or TUI workbench resolves the session through `editorial/ui/`.
7. `editorial/writeback/` lints and imports the approved changes back into authored TypeScript files and review registries.
8. The updated authored state is republished by the same runtime path on the next run.

That loop is the architecture of the editorial subsystem: authored truth, durable review inputs, published runtime, evidence-producing analysis, session-based human review, and controlled writeback.
