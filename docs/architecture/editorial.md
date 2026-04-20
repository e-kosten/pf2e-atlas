# Derived-Tag Editorial Architecture

This document describes the current architecture of the derived-tag editorial subsystem under `src/tags/`. It explains how the existing modules fit together. It does not propose a new tag tree or a new editorial model.

For broader project layering, read [overview.md](./overview.md) and [boundaries.md](./boundaries.md) first. For the non-obvious decisions this document relies on, see:

- [0001 Shared Backend, Separate MCP and Editorial Surfaces](./decisions/0001-shared-backend-separate-surfaces.md)
- [0002 Readonly Ontology and Explicit Storage Boundaries](./decisions/0002-readonly-ontology-and-explicit-storage-boundary.md)
- [0003 Lint-Enforced Derived-Tag Boundaries](./decisions/0003-lint-enforced-derived-tag-boundaries.md)

## Purpose

`src/tags/` exists to support a human-in-the-loop editorial workflow around derived tags:

- define the authored tag ontology and explainable rules
- carry explicit assignments, exemplars, and migration-era compatibility inputs
- inspect the live indexed corpus for missing or over-broad coverage
- review queued decisions in terminal workflows
- write approved changes back into authored TypeScript sources

This subsystem is partly runtime infrastructure and partly editorial tooling. The runtime portion publishes the canonical derived-tag model used by indexing and record derivation. The editorial portion generates evidence, review sessions, and writeback artifacts for humans maintaining that model. The live execution layer now sits under `src/tags/editorial/`.

When evaluating the intended long-term shape of `src/tags/`, treat `legacy-rules/` and `legacy-seed-migrations/` as transitional placeholders only. They exist to encode the old rules format while that content is migrated into the current authored model, not to define the future-state layout for the subsystem.

## Subsystem Shape

```mermaid
flowchart LR
  ontology["`ontology/`
  Authored family/tag hierarchy"]
  rules["`rules/`
  Explainable rule definitions"]
  assignments["`assignments/`
  `reviews/assignment-reviews/`
  `reviews/assignment-memory/`
  Explicit decisions and pending review"]
  exemplars["`exemplars/`
  `reviews/exemplar-reviews/`
  Positive/negative teaching sets"]
  legacy["`legacy-rules/`
  `legacy-seed-migrations/`
  Old rules-format compatibility placeholders"]
  runtime["`runtime/`
  Publish, compile, validate, derive"]
  index["SQLite index + embeddings"]
  discovery["`discovery/`
  Candidate mining and cohorting"]
  evaluation["`evaluation/`
  Evidence, gaps, movement"]
  migration["`editorial/`
  Sessions, review UI, importer"]
  cli["`cli/discovery/`
  `cli/evaluation/`
  `cli/editorial/`
  `cli/shared/`
  Offline entrypoints"]
  tui["TUI workbench"]
  app["`src/app/`
  storage-service + ontology-service"]

  ontology --> runtime
  rules --> runtime
  assignments --> runtime
  exemplars --> runtime
  legacy --> runtime

  runtime --> discovery
  runtime --> evaluation
  runtime --> migration

  index --> discovery
  index --> evaluation
  index --> migration
  index --> app

  discovery --> migration
  evaluation --> migration

  migration --> assignments
  migration --> rules
  migration --> exemplars

  cli --> discovery
  cli --> evaluation
  cli --> migration
  tui --> migration
  app --> tui
```

The important split is:

- authored source trees define the editorial truth
- `runtime/` publishes that truth into a validated, queryable model
- discovery and evaluation read the live index and propose or measure changes
- migration and review workflows turn those proposals into explicit authored edits

## Major Subareas

| Area                         | Owns                                                                                                                                                                 | Consumes                                                                                                 | Produces                                                                                 |
| ---------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------- |
| `runtime/`                   | Published ontology, compiled rules, explicit-assignment index, migration-era legacy-seed index, exemplar publication, tag derivation                                  | Authored ontology, authored rules, explicit assignments, exemplars, legacy compatibility inputs          | `deriveRecordTags`, source-aware derivations, validation helpers, runtime lookup helpers |
| `rules/`                     | Per-category explainable rule definitions and the compiler that normalizes them against the ontology                                                                 | Authored tag names and family scopes from `ontology/`                                                    | Runtime-matchable rule objects                                                           |
| `ontology/`                  | The authored family/tag hierarchy, scope, policy, adjacency, composite structure, and editorial descriptions                                                         | Category manifest and shared type contracts                                                              | Flattenable category ontologies for publication and exploration                          |
| `exemplars/`                 | Curated positive and negative record examples per tag                                                                                                                | Canonical record keys and the authored ontology                                                          | Published exemplar sets and validation input                                             |
| Reviews and explicit storage | `assignments/`, `reviews/assignment-reviews/`, `reviews/assignment-memory/`, `reviews/exemplar-reviews/` carry direct editorial decisions, queued review items, and memory from prior review | The ontology and current editorial policy                                                                | Explicit include/exclude overrides, pending review queues, writeback targets             |
| `discovery/`                 | SQL- and embedding-backed candidate mining, reviewed-record bookkeeping, cohort clustering, evidence normalization                                                   | The live SQLite index, embeddings, published runtime helpers, reviewed selections                        | Candidate cohorts and discovery records that suggest possible new rules or assignments   |
| `evaluation/`                | Evidence reports, gap analysis, and baseline-versus-current movement checks                                                                                          | Discovery records, live index state, runtime exemplar and family lookups                                 | Editorial measurement artifacts used to decide whether a change is worth authoring       |
| `editorial/`                 | Session building, review-screen state, importer, linter, authored-state cloning, runtime review summaries                                                            | Authored source state, runtime publication, review queues, index-backed record loading                   | Human review sessions and on-disk updates to authored TypeScript files                   |
| `cli/discovery/`, `cli/evaluation/`, `cli/editorial/`, `cli/shared/` | Thin offline entrypoints plus shared CLI scope parsing for discovery, evaluation, editorial workflows, and the TUI launcher                                         | Tag/editorial services plus config/index opening                                                         | Human-readable reports and local workflows                                               |

## Runtime Layer

`src/tags/runtime/index.ts` is the publication boundary for the subsystem. It assembles and exports the pieces that the rest of the application can safely consume:

- publish the authored ontology into flattened family/tag registries
- compile authored rules against that ontology
- combine authored rules, explicit assignments, and the remaining migration-era legacy inputs into one derivation path
- publish exemplar registries and validate them against canonical records
- expose stable helpers such as `deriveRecordTags`, `getDerivedTagFamilyTags`, and `getVariantInheritableTags`

Architecturally, `runtime/` is where the editorial source files stop being category-specific authoring data and become a coherent derived-tag backend.

## Authored Knowledge

The authored source of truth is spread across several narrow trees:

- `ontology/`: defines families, tags, scope, descriptions, adjacency, composite rules, and inheritance policy
- `rules/`: defines explainable rule blocks that compile into matcher-friendly runtime rules
- `assignments/`: captures explicit per-record overrides that should win without rediscovering the same conclusion
- `exemplars/`: keeps small positive and negative teaching sets for discovery and review workflows
- `legacy-rules/` and `legacy-seed-migrations/`: migration-only compatibility placeholders that preserve the old rules format while it is being retired

These trees are intentionally category-sharded (`affliction`, `creature`, `equipment`, `hazard`, `spell`) so editorial work can stay local while the runtime layer recombines them through the shared manifest.

## Discovery and Evaluation Loop

Discovery and evaluation are not the tagging runtime. They are the evidence loop around it.

`discovery/` loads record slices from the SQLite index, embeddings, traits, references, and reviewed-record history to answer questions like:

- which untagged records cluster near current exemplars?
- which cohorts look ruleable versus manual-only?
- which reviewed records were intentionally excluded and why?

`evaluation/` converts that evidence into decision support:

- `evidence-analyzer.ts` explains common tokens, phrases, traits, and references
- `gap-evaluator.ts` finds likely missing coverage for an existing tag
- `movement-evaluator.ts` compares baseline and current indexes to catch regressions or weak gains

Those modules are deliberately advisory. They surface evidence for editors, but they do not directly rewrite authored files.

## Migration and Review Loop

`editorial/` is the execution layer for editorial change.

It owns four distinct responsibilities:

- clone the imported authored registries into a mutable in-memory working state
- build review sessions from pending queues, legacy-input migration work, exemplar cleanup, or legacy-rule takeover work
- present those sessions in CLI and TUI review flows
- lint and import approved sessions back into source files under `src/tags/`

Two internal files matter architecturally:

- `authored-state.ts` is the explicit storage boundary for editable editorial state during a session
- `runtime-state.ts` republishes the current working state into queue summaries, pending-assignment views, and a published ontology snapshot for the workbench

This separation keeps the workbench from mutating imported modules in place while still letting the review UI operate against a coherent current snapshot.

## Review Registries and Explicit State

The review trees are easy to miss because they are small, but they are a key architectural seam:

- `reviews/assignment-reviews/` holds pending include/exclude decisions that still need resolution
- `reviews/assignment-memory/` keeps prior reasoning that should remain visible during future review
- `reviews/exemplar-reviews/` holds pending exemplar polarity or keep/drop decisions

These registries are not just scratch data. They are durable editorial inputs that the editorial session builder reads when it constructs the current queue and that writeback code preserves alongside authored rules, assignments, and exemplars.

## CLI and TUI Surfaces

The CLI files under `src/tags/cli/` are intentionally thin and grouped by concern:

- `src/tags/cli/discovery/` for candidate-mining workflows
- `src/tags/cli/evaluation/` for evidence and movement reports
- `src/tags/cli/editorial/` for session creation, review, import, and the workbench launcher
- `src/tags/cli/shared/` for shared helpers such as scope parsing

- parse scope arguments and validation flags
- open the configured SQLite index
- call one discovery, evaluation, or migration service
- print a report or launch the TUI

The special case is `src/tags/cli/editorial/derived-tag-migration-workbench.ts`, which launches the terminal app rather than implementing review logic itself. The actual workbench composition happens through `src/tui/app-services.ts`, which wires:

- shared application runtime and `Pf2eDataService`
- `src/app/storage-service.ts` for explicit index opening
- `src/app/ontology-service.ts` for the readonly explorer model
- `src/tags/editorial/workbench-controller.ts` for tag-review workflows

That split is important. The editorial TUI is a surface over shared backend and migration services, not a second copy of the editorial logic.

## Boundaries To Preserve

The current codebase already encodes several intended boundaries:

- outside `src/tags`, callers should normally import derived-tag functionality through `src/tags/index.ts` or a higher-level app service instead of leaf tag modules
- ontology browsing is assembled through `src/app/ontology-service.ts` and treated as a readonly model, not mutable shared UI state
- direct SQLite opening is explicit through the application storage service or justified offline tag tooling, not scattered through unrelated feature modules
- CLI scope parsing is centralized in `src/tags/cli/shared/search-scope-args.ts`
- discovery and evaluation can recommend changes, but migration and import own writeback into authored files

Those rules are documented in `docs/architecture/boundaries.md` and reinforced by ESLint restrictions where the shared abstraction is already considered stable.

## End-to-End Editorial Flow

The steady-state maintenance loop looks like this:

1. Editors maintain authored ontology, rules, assignments, exemplars, and review registries in `src/tags/`, with queued review inputs under `src/tags/reviews/`.
2. `runtime/` publishes those inputs into the canonical derivation model used during indexing and other derived-tag lookups.
3. Discovery and evaluation inspect the current SQLite index and embeddings to find missing coverage, overfit clusters, and movement against a baseline.
4. Migration builds a review session from those signals or from pending queues.
5. The CLI or TUI workbench resolves the session and imports the approved changes back into authored TypeScript files.
6. The updated authored state is then republished by the same runtime path on the next run.

That loop is the architecture of the editorial subsystem: explicit authored truth, published runtime, evidence-producing analysis, and human-approved writeback.
