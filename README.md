# Pathfinder 2E Foundry MCP

Offline Pathfinder 2E search, reference, and editorial workbench built on a local clone of the [Foundry PF2E repository](https://github.com/foundryvtt/pf2e). It supports querying and exploring Foundry PF2E data locally through either an MCP server or a TUI.

It provides several connected surfaces over the local Foundry PF2E data:
- a stdio MCP server for external clients and agents
- a TUI for exploration, search, and editorial workflows
- local indexing, search-semantics, and derived-tag tooling for maintaining and refining the corpus

## Capabilities

With it, you can:

- look up detailed PF2E records by name and canonical key
- search across the PF2E corpus with category-first boundaries, structured filters, and hybrid retrieval
- connect external clients and agents to PF2E search and rules tools over MCP
- explore the indexed corpus interactively through the TUI
- work with Pathfinder-native search semantics, metadata filters, and linked-rule traversal
- run derived-tag discovery, review, migration, and evaluation workflows for corpus maintenance

## Quick Start

Before you start:

- install Node.js 20+
- clone the Foundry PF2E repo into `vendor/pf2e`

Set up the local runtime:

```bash
npm install
git clone https://github.com/foundryvtt/pf2e.git vendor/pf2e
npm run refresh-external
npm run build
```

## Run

Build once, then launch either main app surface from the repo root:

```bash
npm run tui
npm run mcp
```

`npm run tui` runs the built terminal workbench from `dist/tags/cli/editorial/derived-tag-migration-workbench.js`.
`npm run mcp` runs the built stdio MCP server from `dist/index.js`.

For deeper local developer and editorial tooling, use the script surfaces under [`scripts/`](./scripts/package.json) and [`src/tags/cli/`](./src/tags/cli/package.json). See [CONTRIBUTING.md](./CONTRIBUTING.md) for the command layout.

## Data And Index

The application reads PF2E data from `vendor/pf2e` by default and builds a local SQLite index for querying, exploration, and editorial work.
That index is cached and reused across restarts. When the PF2E source, embedding model, or index schema changes, rebuild it explicitly with `npm run refresh-index` or `npm run refresh-external`.

Normal startup is offline-only. It does not refresh the vendored PF2E clone, download embedding assets, or rebuild the SQLite index for you.
Refresh external dependencies explicitly when needed:

```bash
npm run refresh-data
npm run refresh-embeddings
npm run refresh-index

# or run the full refresh flow at once
npm run refresh-external
```

`refresh-index` rebuilds the local SQLite cache from the already-prepared PF2E checkout and embedding assets. Application startup expects that index to already exist and be current.

Run `refresh-index` again whenever:

- `refresh-data` changed the vendored PF2E checkout
- `refresh-embeddings` changed the configured embedding model or revision
- you pulled code that changed the index schema

If startup is missing a prepared index or detects a stale one, it fails fast with an error telling you to run `npm run refresh-index` or `npm run refresh-external`.

## MCP Client Config

Example MCP client entry:

```json
{
  "mcpServers": {
    "pathfinder-2e-foundry": {
      "command": "node",
      "args": ["/Users/<user>/projects/pathfinder-mcp/pathfinder-2e-foundry-mcp/dist/index.js"]
    }
  }
}
```

The MCP client should point at the built server entrypoint. Build first with `npm run build`, then use `npm run mcp` for manual local runs.

## Configuration

The normal workflow is to keep the PF2E data under `vendor/pf2e`, but an alternate path can be supplied if needed.

You can also override the data path with:

- `PF2E_DATA_PATH`
- `--data-path /path/to/pf2e`

The SQLite index path defaults to `.cache/pf2e-index.sqlite` under the current working directory. You can override it with:

- `PF2E_INDEX_PATH`
- `--index-path /path/to/index.sqlite`

The embedding cache path defaults to `.cache/hf-models`. You can override it with:

- `PF2E_EMBEDDING_CACHE_PATH`
- `--embedding-cache-path /path/to/hf-models`

If you want to bypass local HF embeddings entirely, set:

- `PF2E_EMBEDDING_PROVIDER=hash`

That skips the HF model requirement, but you still need a prepared SQLite index for startup.

## MCP And Search

The MCP server is read-only and exposes a category-first search surface over the local PF2E corpus.

- Use `pf2e_lookup` and `pf2e_lookup_many` for exact-name lookups.
- Use `pf2e_search` for ranked retrieval with `searchProfile: "lexical" | "balanced" | "concept"`.
- Use `pf2e_get_search_semantics` and `pf2e_list_filter_values` to discover categories, subcategories, tags, and live filter values before building structured queries.
- Prefer `category`, `subcategory`, `scopes`, numeric bounds, and typed `metadata` predicates over raw Foundry record-type fields.

For the full MCP tool catalog, response field reference, and detailed search behavior notes, see [docs/mcp-and-search.md](./docs/mcp-and-search.md).

## Further Reading

- [CONTRIBUTING.md](./CONTRIBUTING.md) for contributor workflow, developer commands, and repo layout
- [docs/mcp-and-search.md](./docs/mcp-and-search.md) for MCP tool and search reference
- [docs/architecture](./docs/architecture/overview.md) for architecture, boundaries, and subsystem docs
