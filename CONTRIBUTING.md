# Contributing

## Branching

This repo uses a simple trunk-based workflow:

- `main` is the primary branch
- create short-lived feature branches from `main`
- merge back to `main` after validation

Recommended branch names:

- `feat/<topic>`
- `fix/<topic>`
- `docs/<topic>`
- `chore/<topic>`

Examples:

- `feat/encounter-tools`
- `fix/search-ranking`
- `docs/codex-setup`

## Development

Install dependencies and build:

```bash
npm install
npm run build
```

Run the test suite:

```bash
npm test
```

Run the stdio MCP server locally:

```bash
PF2E_DATA_PATH=~/projects/pathfinder-mcp/pf2e npm run dev
```

## Validation Before Commit

Run these before opening a branch for review or merging back to `main`:

```bash
npm run build
npm test
```
