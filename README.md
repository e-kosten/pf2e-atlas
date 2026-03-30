# Pathfinder 2E Foundry MCP

Read-only MCP server for Pathfinder 2E data exported from a local Foundry PF2E system checkout.

## What It Does

- Looks up detailed PF2E records by name
- Lists available compendium packs and pack metadata
- Lists records within a pack with filters
- Searches across all packs by name and structured filters
- Returns the original Foundry JSON for detailed retrieval

The server uses `stdio` in v1 and reads the PF2E data from a configurable local path.

## Requirements

- Node.js 20+
- A local PF2E Foundry checkout or export containing `system.pf2e.json`

For your current setup, the data path is:

```text
~/projects/pathfinder-mcp/pf2e
```

## Setup

```bash
npm install
npm run build
```

## Configuration

Set the data path with either:

- `PF2E_DATA_PATH`
- `--data-path /path/to/pf2e`

Example:

```bash
PF2E_DATA_PATH=~/projects/pathfinder-mcp/pf2e npm run dev
```

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
      ],
      "env": {
        "PF2E_DATA_PATH": "/Users/<user>/projects/pathfinder-mcp/pf2e"
      }
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
- `pf2e_get_record`

## Notes

- The server is read-only.
- Search in v1 is name-driven plus structured filters, not full-text description search.
- The transport layer is isolated so Streamable HTTP can be added later without rebuilding the data/index layer.
