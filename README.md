# PF2e Atlas

PF2e Atlas is a local Pathfinder 2E search and reference runtime built from the [Foundry PF2E repository](https://github.com/foundryvtt/pf2e). The Rust `atlas` CLI is the primary local interface in this branch: it installs and repairs local data, builds the SQLite artifact, validates readiness, and exposes record lookup, resolution, and search commands.

## Capabilities

With `atlas`, you can:

- fetch and maintain the local Foundry PF2E source checkout
- build and validate a local SQLite artifact for PF2E records
- look up detailed PF2E records by canonical key
- resolve names and verified aliases to canonical records
- search and list records with structured filters
- validate full semantic readiness, base record readiness, or focused embedding/vector readiness

## Quick Start

Before you start, install Rust and Cargo with [rustup](https://rustup.rs/) or your platform package manager.

Install the local CLI from this clone:

```bash
cd rust
cargo install --path crates/atlas-cli --locked
```

Run the standard first-time setup:

```bash
atlas setup
```

`atlas setup` resolves the default paths, fetches or updates the Foundry PF2E source checkout, prepares the configured embedding model cache, builds or repairs the local SQLite artifact, and validates the result. Embeddings are required by default because semantic search depends on them.

For a faster record-only setup, use:

```bash
atlas setup --no-embeddings
```

That produces a base artifact suitable for `record get`, `record resolve`, and filter-only listing, but not semantic search.

## Try It

After setup, commands use the resolved default artifact path automatically:

```bash
atlas record get actionspf2e:1kGNdIIhuglAjIp9
atlas record get equipment-srd:s1vB3HdXjMigYAnY
atlas record resolve "Treat Wounds" --filter-json '{"kind":"pack","value":"actionspf2e"}'
atlas index validate
```

Useful setup and validation variants:

```bash
atlas setup --check
atlas setup --offline
atlas index validate --no-embeddings
atlas index validate --embeddings-only
```

`atlas setup --check` reports readiness and planned actions without writing local runtime files. `atlas setup --offline` prevents network-backed source updates and embedding model preparation. `atlas index validate` validates full semantic readiness by default; `--no-embeddings` validates only the base artifact, and `--embeddings-only` runs focused embedding/vector diagnostics.

## Paths And Data

`atlas` uses the Rust runtime path resolver:

- `--path-mode auto` is the default.
- Inside this repository, auto mode uses repo-local paths:
  - source: `vendor/pf2e`
  - embedding model cache: `.cache/hf-models`
  - SQLite artifact: `.cache/pf2e-rust-index.sqlite`
- Outside a repository checkout, auto mode uses platform user cache paths under `pf2e-atlas`.
- `--path-mode repo` requires repo-local paths.
- `--path-mode user` forces platform user cache paths.

Direct command flags override resolved paths for that command, such as:

- `--source /path/to/pf2e`
- `--output /path/to/pf2e-rust-index.sqlite`
- `--index /path/to/pf2e-rust-index.sqlite`
- `--embedding-cache-path /path/to/hf-models`

The local SQLite artifact is reusable. Setup rebuilds or repairs it when validation fails, when the source signature is stale, when embedding metadata does not match the selected model, or when the embedding-unit policy changes.

## JSON Output

Most commands support `--json`. Successful command payloads are under `data`, and command failures are under `error`.

Artifact validation against an invalid artifact is still a successful command invocation: it returns `status: "ok"` with `data.valid: false` and exits with code `3`.

Example:

```bash
atlas index validate --json
```

## Manual Index Commands

Standard users should run `atlas setup`. Manual index commands remain available for development and diagnostics:

```bash
cd rust
cargo run -p atlas-cli -- index analyze --json
cargo run -p atlas-cli -- index build --no-embeddings --json
```

Use Cargo's release profile for ingest or search performance measurements:

```bash
cargo run --release -p atlas-cli -- index analyze --source ../vendor/pf2e --json
```

## Transitional Node Surfaces

The TypeScript/Node MCP server, TUI, and editorial tooling still exist in this repository while the Rust runtime is being promoted. Those surfaces use the older Node setup and index flow.

For the Node surfaces:

```bash
npm install
npm run refresh-external
npm run build
npm run tui
npm run mcp
```

`npm run tui` runs the built terminal workbench from `dist/tags/cli/editorial/derived-tag-migration-workbench.js`.
`npm run mcp` runs the built stdio MCP server from `dist/index.js`.

The Node runtime remains offline-only at startup. It expects its own prepared SQLite index and embedding assets. The Rust `atlas setup` flow described above is the intended first-run path for the Rust CLI artifact, not a replacement for the current Node MCP server configuration.

## MCP Client Config

The current MCP server is still the TypeScript stdio server. Build it first with `npm run build`, then configure the client to point at the built server entrypoint:

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

Keep the full path as one JSON/TOML string or one shell argument; a line break inside the path makes Node try to load only the prefix before the break.

## Further Reading

- [rust/README.md](./rust/README.md) for Rust workspace layout, validation commands, and CLI details
- [CONTRIBUTING.md](./CONTRIBUTING.md) for contributor workflow, developer commands, and repo layout
- [docs/mcp-and-search.md](./docs/mcp-and-search.md) for the current TypeScript MCP tool and search reference
- [docs/architecture](./docs/architecture/overview.md) for architecture, boundaries, and subsystem docs

## Contact

For questions about PF2e Atlas, use the [GitHub issue tracker](https://github.com/e-kosten/pf2e-atlas/issues).

## Paizo Community Use Notice

PF2e Atlas uses trademarks and/or copyrights owned by Paizo Inc., used under Paizo's Community Use Policy (paizo.com/licenses/communityuse). We are expressly prohibited from charging you to use or access this content. PF2e Atlas is not published, endorsed, or specifically approved by Paizo. For more information about Paizo Inc. and Paizo products, visit paizo.com.
