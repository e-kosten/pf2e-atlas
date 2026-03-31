# Pathfinder 2E Foundry MCP

Read-only MCP server for Pathfinder 2E data from a vendored local Foundry PF2E checkout.

## What It Does

- Looks up detailed PF2E records by name
- Lists available compendium packs and pack metadata
- Lists records within a pack with filters
- Searches across all packs by name and structured filters
- Follows linked rules references for actions, conditions, spells, and other rules text
- Supports user-facing search profiles: `lookup`, `balanced`, and `concept`
- Returns the original Foundry JSON for detailed retrieval

The server uses `stdio` in v1, reads the PF2E data from `vendor/pf2e` by default, and builds a local SQLite index for querying.
That index is cached and reused across restarts. When the PF2E source, embedding model, or index schema changes, rebuild it explicitly with `npm run refresh-index` or `npm run refresh-external`.

## Requirements

- Node.js 20+
- A local clone of the Foundry PF2E system repo under `vendor/pf2e`

## Setup

```bash
npm install
git clone https://github.com/foundryvtt/pf2e.git vendor/pf2e
npm run refresh-external
npm run build
```

## Data Checkout

Clone the PF2E repo into the vendored data path:

```bash
git clone https://github.com/foundryvtt/pf2e.git vendor/pf2e
```

Normal MCP startup is offline-only. It does not run `git pull`, it does not download embedding assets from Hugging Face, and it does not rebuild the SQLite index.
Refresh external dependencies explicitly before launching the server:

```bash
npm run refresh-data
npm run refresh-embeddings
npm run refresh-index
# or both at once
npm run refresh-external
```

`refresh-index` rebuilds the local SQLite cache from the already-prepared PF2E checkout and embedding assets. Normal MCP startup expects that index to already exist and be current.

Run `refresh-index` again whenever:

- `refresh-data` changed the vendored PF2E checkout
- `refresh-embeddings` changed the configured embedding model or revision
- you pulled code that changed the index schema

If startup is missing a prepared index or detects a stale one, it fails fast with an error telling you to run `npm run refresh-index` or `npm run refresh-external`.

An alternate PF2E data path can still be supplied with `--data-path /path/to/pf2e` if needed, but the normal workflow is to keep the data under `vendor/pf2e`.

The SQLite index path defaults to `.cache/pf2e-index.sqlite` under the current working directory. You can override it with:

- `PF2E_INDEX_PATH`
- `--index-path /path/to/index.sqlite`

The embedding cache path defaults to `.cache/hf-models`. You can override it with:

- `PF2E_EMBEDDING_CACHE_PATH`
- `--embedding-cache-path /path/to/hf-models`

If you want to bypass local HF embeddings entirely, set:

- `PF2E_EMBEDDING_PROVIDER=hash`

That skips the HF model requirement, but you still need a prepared SQLite index for startup.

## Development Workflow

This repo uses a simple trunk-based Git workflow:

- keep `main` as the primary branch
- create short-lived branches for work such as `feat/<topic>` or `fix/<topic>`
- validate with `npm run build` and `npm test` before merging

See [CONTRIBUTING.md](./CONTRIBUTING.md) for the local workflow details.

## MCP Client Config

Example MCP client entry:

```json
{
  "mcpServers": {
    "pathfinder-2e-foundry": {
      "command": "node",
      "args": [
        "/Users/<user>/projects/pathfinder-mcp/pathfinder-2e-foundry-mcp/dist/index.js"
      ]
    }
  }
}
```

## Tools

- `pf2e_list_categories`
- `pf2e_get_pack_metadata`
- `pf2e_list_records`
- `pf2e_get_search_semantics`
- `pf2e_search`
- `pf2e_lookup`
- `pf2e_get_rules_context`
- `pf2e_get_record`

Search and list responses include:

- `searchProfile` as the user-facing retrieval intent used for the result set
- `category` and `subcategories` as the primary product-facing search boundaries
- `rawRecordType` for internal Foundry traceability when needed
- `descriptionText` when available
- `hasDescription` for filtering and ranking
- `descriptionSnippet` for lightweight discovery
- `sourceCategory` to distinguish core, rules, adventure, and unknown sources
- `searchExplain` on records when `pf2e_search` is called with `explain: true`
- `explain.query` details for literal query normalization when `pf2e_search` is called with `explain: true`

## Notes

- The server is read-only.
- Search is category-first. Use `category` and optional `subcategory` as the main boundaries instead of raw Foundry `recordType`, `documentType`, or `itemCategory`.
- `pf2e_search` supports `searchProfile: "lookup" | "balanced" | "concept"` as the primary user-facing retrieval control.
- `pf2e_search` defaults to the `balanced` profile when `themeQuery` is present and `searchProfile` is omitted.
- `pf2e_get_search_semantics` is the primary discovery surface for categories, subcategories, Pathfinder-native tags, and supported filters.
- Search now uses a local SQLite index with:
  - shared structured filters
  - FTS-backed lexical search
  - hybrid semantic reranking over filtered candidates
  - explicit name, trait, metadata, and description scoring with optional explain output
  - no server-side query expansion or hidden semantic-tag inference
- Semantic search is implemented with local application-side vector scoring after SQLite hard filters.
- When `hf-local` is configured, the embedding model must already be prepared locally with `npm run refresh-embeddings` or `npm run refresh-external`.
- The SQLite index must already be prepared locally with `npm run refresh-index` or `npm run refresh-external`.
- ANN and SQLite vector extensions are intentionally not required in the current implementation.
- The transport layer is isolated so Streamable HTTP can be added later without rebuilding the data/index layer.
- The vendored PF2E checkout under `vendor/pf2e` is intentionally ignored by this repo's Git history.
