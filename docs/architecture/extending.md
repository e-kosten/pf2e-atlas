# Extending The Architecture

This guide is for future human and AI editors adding features to the repository. The goal is not just to place code somewhere that works. The goal is to place it in the owning layer, thread it through the right facade, and tighten the boundary when the new path is meant to be the standard route.

Read `docs/architecture/overview.md` first, then `docs/architecture/boundaries.md`, then start from the relevant composition root instead of from a random leaf file.

## How To Choose The Owning Layer

Pick the lowest layer that can own the behavior without depending on a higher-level concern.

| If the change is primarily...                                                           | Start here                    | Do not start here                                |
| --------------------------------------------------------------------------------------- | ----------------------------- | ------------------------------------------------ |
| MCP wire schema, tool registration, or response shaping                                 | `src/server/`                 | `src/domain/` or low-level SQL helpers           |
| Terminal workflow, screen behavior, or prompt orchestration                             | `src/tui/`                    | `src/server/` or direct storage/runtime assembly |
| Cross-surface runtime composition or app-scoped facade wiring                           | `src/app/`                    | individual screens or tool registration files    |
| Backend retrieval, index-backed record access, rule graph, or reusable search execution | `src/data/` and `src/search/` | `src/server/` or `src/tui/`                      |
| Shared types, category vocabularies, ontology contracts, or metadata semantics          | `src/domain/`                 | transport/UI layers                              |
| Derived-tag authoring, migration, or editorial internals                                | `src/tags/`                   | non-tag callers outside a facade                 |

Useful heuristics:

- If both MCP and TUI will need it, it probably does not belong in `src/server/` or `src/tui/`.
- If the code needs `DatabaseSync`, it probably belongs in `src/data/`, `src/app/storage-service.ts`, or an approved CLI entrypoint.
- If the code mostly translates inputs and outputs around an existing service call, it probably belongs in a thin surface layer.
- If the code defines vocabulary rather than behavior, it probably belongs in `src/domain/`.
- If a non-tag caller needs a domain type, import the concrete `src/domain/*` owner file instead of defaulting to `src/domain/index.ts`.
- If a helper clearly belongs to `app`, `data`, or `search`, keep it there instead of extending `src/shared/`.

## When To Add A Facade

Add or extend a facade when callers should not know about the lower-level implementation details.

Typical signals that a facade is warranted:

- more than one caller needs the same behavior
- callers should not manage runtime composition, storage opening, or backend service assembly themselves
- you want to refactor internal modules later without changing the call sites
- the behavior crosses a layer boundary and needs one stable entrypoint

Existing examples:

- `Pf2eDataService` is the main backend facade for lookup, search, list, and rule-graph work
- `createPf2eApplicationOntologyService` is the app-layer ontology facade
- `createPf2eTerminalSearchService` is the TUI-facing search facade
- `src/tui/app-services.ts` is the TUI composition root and service bundle
- `src/tags/index.ts` is the preferred non-tag entrypoint for tag functionality

Do not add a facade just to hide a one-off helper used in one file. Add one when you are trying to define the standard entrypoint for a concern.

## When To Add A Lint Rule

Add or tighten a lint rule when the new facade or pathway is no longer optional.

Use this checklist:

1. Is there now a preferred shared abstraction?
2. Would bypassing it create duplicate logic, lifecycle bugs, or architectural drift?
3. Is the abstraction stable enough that future code should be forced through it?
4. Is the exception list short and easy to explain?

Choose the rule shape that matches the boundary:

- Use `eslint-local-rules.js` for low-level operations that should be rare everywhere, such as raw `JSON.parse`, `DatabaseSync`, or direct terminal event decoding.
- Use `no-restricted-imports` in `eslint.config.js` for layer ownership, facade routing, and "do not import this leaf from that layer" constraints.
- Use `no-restricted-syntax` when the problem is an API pattern or workflow shape rather than a specific import path.

If the abstraction is still exploratory, document the intended direction first and wait on enforcement. Lint rules should lock in a stable boundary, not guess at one.

## Practical Workflow For Adding A Feature

Use this sequence when you add a new capability:

1. Identify the user-facing surface.
2. Trace the relevant composition root:
   - MCP work starts from `src/index.ts` and `src/server/`
   - TUI work starts from `src/tui/app-services.ts`
3. Choose the owning layer for the reusable behavior.
4. Implement or extend that reusable behavior in the owning layer first.
5. Thread the new behavior through the appropriate facade.
6. Add the thin surface adapter in `src/server/` or `src/tui/`.
7. Decide whether the new path is now mandatory enough to deserve lint enforcement.
8. Add or update tests for the layer that owns the behavior.

For AI editors in particular: do not start by copying the nearest leaf file that looks similar. Start from the composition root and the facade boundary. That avoids introducing a parallel path that works locally but violates the repo's layering rules.

## Concrete Examples

### Adding A New MCP Tool

A new MCP tool usually means:

1. add or extend backend behavior behind `Pf2eDataService`
2. if needed, add the lower-level implementation in `src/data/backend/` or `src/search/`
3. register the tool in the relevant `src/server/register-*.ts` module
4. shape the response in `src/server/presenters.ts` or nearby server helpers

Do not import `src/search/sql.ts`, `src/data/record-queries.ts`, or `src/data/schema.ts` from a server registration module. The lint config blocks that on purpose. If the server layer needs new data, the fix is to extend the backend facade first.

### Adding A New TUI Feature

A new TUI workflow or screen usually means:

1. expose the needed service through `src/tui/app-services.ts`
2. if the behavior is TUI-specific but reusable across TUI files, add or extend a TUI-facing facade such as `src/tui/search/service.ts`
3. keep low-level prompt, input, and navigation behavior inside existing routers, controllers, or shared screen helpers

Do not:

- call `loadPf2eApplicationRuntime()` from an arbitrary TUI feature file
- construct `DatabaseSync` in TUI code
- import raw migration internals when `app-services` should compose them
- bypass shared interaction helpers by decoding terminal events inline

If you need a new kind of reusable interaction flow, add the helper first, then consider a lint rule so future screens use it consistently.

### Making A Backend Search Change

If you are changing search ranking, query analysis, browse/search execution, or search window behavior:

- put reusable search mechanics in `src/search/`
- put backend coordination in `src/data/backend/search-service.ts` or adjacent backend services
- surface the capability through `Pf2eDataService`
- let MCP and TUI consume that service rather than reproducing the logic

If the change creates a new canonical search pathway that should replace direct legacy imports, add a `no-restricted-imports` rule so future search or server code cannot bypass it.

### Making An Ontology Change

If you are changing browse models or ontology assembly:

- own the behavior in `src/app/ontology/` and `src/app/ontology-service.ts`
- keep returned ontology models readonly
- move any shared vocabulary or contracts into `src/domain/` if they are no longer ontology-specific

If the change is only about TUI presentation of ontology nodes, keep it in `src/tui/ontology-explorer/` and do not mutate shared ontology nodes to carry UI state. Derived or cached UI state should live beside the UI helper that owns it.

## A Short Decision Test

Before you finish a feature, ask:

- Would another surface need to copy this logic to reuse it?
- Did I add a new direct dependency on storage, runtime assembly, or TUI framework details?
- Did I put vocabulary in a behavior layer, or behavior in a transport layer?
- Is the preferred path now clear enough that lint should enforce it?

If the answer to the first two questions is yes, the code probably needs to move down a layer or behind a facade before it is finished.
