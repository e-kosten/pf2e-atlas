# 0019 Rust CLI Search And Record Surface

## Status

Accepted

## Context

The TypeScript MCP product exposes separate lookup, search, and browse/list tools because those tools map cleanly to the internal `SearchRequest` modes:

- lookup is name/alias-oriented structured retrieval
- search is ranked text retrieval over lexical, semantic, or hybrid signals
- browse/list is filter-only structured retrieval

That distinction is useful in the runtime, but it is not the clearest local CLI product model. A regular CLI user should not have to choose between named lookup, full-text search, semantic search, and filter-only browse as separate product surfaces before they know which retrieval behavior they need.

The Rust CLI should keep strict record identification available for agent workflows, but the default search surface should feel like one search command:

- text search should boost strong name and alias matches before other lexical or semantic matches
- filter-only invocation should behave as deterministic listing
- FTS, vector, and hybrid behavior should be selectable as retrieval or diagnostic options rather than separate default product paths
- strict name resolution should remain available when a caller needs a single record or a clear miss

## Decision

Separate the Rust CLI by user intent and guarantee, not by backend retrieval implementation.

Use the `record` namespace for exact record identification and fetching:

```bash
atlas record get <record-key> --json
atlas record resolve "Treat Wounds" --json
```

Use the `search` namespace for result-set retrieval:

```bash
atlas search "low level healing spell" --json
atlas search "Treat Wounds" --json
atlas search --filter-json '<canonical-filter-json>' --json
atlas search --retrieval fts "Treat Wounds" --json
atlas search --retrieval vector "healing magic" --json
atlas search --retrieval hybrid "low level healing spell" --json
```

`atlas search` is the normal user-facing search entrypoint. When text is present, it may combine name/alias, FTS, vector, and hybrid signals according to the active retrieval path and fusion controls. Strong name and verified alias matches should rank ahead of broader text or semantic matches. When text is absent and filters are present, it routes to deterministic filter-only list behavior using the same structured retrieval core.

`atlas record resolve` is a stricter record-identification command. It resolves display names, normalized names, verified source-backed aliases, and exact full variant names. Its default behavior should be conservative: return a strong match or a clear nonzero miss/ambiguity diagnostic. Fuzzy matching and variant-family expansion are opt-in behaviors, such as `--fuzzy`, `--include-variants`, or `--alternatives`, not silent defaults.

`atlas record get` is exact key retrieval. It should not perform search or name matching.

The runtime should still preserve the internal distinctions:

- filter-only structured retrieval
- strict record resolution
- ranked text search
- FTS, vector, and hybrid retrieval paths

Those distinctions belong behind the CLI contract unless the caller explicitly asks for a retrieval or diagnostic mode.

## Consequences

The Rust migration plan should not implement a top-level `atlas lookup` command as the primary user-facing surface. If a convenience alias is ever added, it should delegate to `atlas record resolve` and not create separate backend semantics.

Phase 5 should be reframed around record retrieval and structured search foundation:

- exact key retrieval through `atlas record get`
- strict name/alias resolution through `atlas record resolve`
- shared record detail output for `summary`, `standard`, and `full`
- filter-only `atlas search` behavior that routes to deterministic list mode when no text is supplied

Later ranked search work should extend the same `atlas search` command rather than adding separate default commands for FTS and semantic search. Power-user retrieval selection can expose FTS, vector, and hybrid behavior as flags.

The CLI can still use internal Rust request models that resemble browse, lookup, and search. The product surface should present record operations and search operations instead of exposing those internal modes as peer commands.

This decision also affects skills and examples: agent guidance should teach `record get` for known keys, `record resolve` for strict names, and `search` for result sets. It should not tell ordinary users to pick separate semantic or lexical commands unless they are tuning retrieval behavior.
