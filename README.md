# Pathfinder 2E Foundry MCP

Read-only MCP server for Pathfinder 2E data from a vendored local Foundry PF2E checkout.

## What It Does

- Looks up detailed PF2E records by name
- Lists available compendium packs and pack metadata
- Lists records within a pack with filters
- Searches across all packs by name and structured filters
- Follows linked rules references for actions, conditions, spells, and other rules text
- Supports structured, lexical, and hybrid search modes
- Returns the original Foundry JSON for detailed retrieval

The server uses `stdio` in v1, reads the PF2E data from `vendor/pf2e` by default, and builds a local SQLite index for querying.
That index is cached and reused across restarts until the underlying PF2E source changes or the index schema version changes.

## Requirements

- Node.js 20+
- A local clone of the Foundry PF2E system repo under `vendor/pf2e`

## Setup

```bash
npm install
git clone https://github.com/foundryvtt/pf2e.git vendor/pf2e
npm run build
```

## Data Checkout

Clone the PF2E repo into the vendored data path:

```bash
git clone https://github.com/foundryvtt/pf2e.git vendor/pf2e
```

On startup, the MCP server runs a best-effort `git pull --ff-only` in `vendor/pf2e` before indexing the data. If refresh fails, startup continues with the existing checkout and reports a warning on stderr.

You can also refresh the vendored checkout manually:

```bash
npm run refresh-data
```

An alternate PF2E data path can still be supplied with `--data-path /path/to/pf2e` if needed, but the normal workflow is to keep the data under `vendor/pf2e`.

The SQLite index path defaults to `.cache/pf2e-index.sqlite` under the current working directory. You can override it with:

- `PF2E_INDEX_PATH`
- `--index-path /path/to/index.sqlite`

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
- `pf2e_search`
- `pf2e_lookup`
- `pf2e_get_rules_context`
- `pf2e_get_record`

Search and list responses include:

- `descriptionText` when available
- `hasDescription` for filtering and ranking
- `descriptionSnippet` for lightweight discovery
- `sourceCategory` to distinguish core, rules, adventure, and unknown sources

## Notes

- The server is read-only.
- Search now uses a local SQLite index with:
  - shared structured filters
  - FTS-backed lexical search
  - hybrid semantic reranking over filtered candidates
- Semantic search is implemented with local application-side vector scoring after SQLite hard filters.
- ANN and SQLite vector extensions are intentionally not required in the current implementation.
- The transport layer is isolated so Streamable HTTP can be added later without rebuilding the data/index layer.
- The vendored PF2E checkout under `vendor/pf2e` is intentionally ignored by this repo's Git history.
