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

## Commit Messages

Use Conventional Commits for all changes.

Format:

```text
type(scope): summary
```

Scope is optional:

```text
type: summary
```

Recommended types for this repo:

- `feat` for new MCP capabilities or data features
- `fix` for bug fixes or behavior corrections
- `docs` for README or contribution guide updates
- `refactor` for internal code restructuring without behavior changes
- `test` for test-only changes
- `chore` for maintenance tasks, scripts, or repo setup

Examples:

- `feat(search): add publication title filtering`
- `fix(lookup): prefer canonical action packs over macro packs`
- `docs(readme): document vendored PF2E checkout`
- `chore(vendor): add refresh script for PF2E data`

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
npm run dev
```

## Validation Before Commit

Run these before opening a branch for review or merging back to `main`:

```bash
npm run build
npm test
```

## Vendored PF2E Data

This repo expects a separate PF2E checkout under `vendor/pf2e`.

Initial clone:

```bash
git clone https://github.com/foundryvtt/pf2e.git vendor/pf2e
```

Manual refresh:

```bash
npm run refresh-data
```
