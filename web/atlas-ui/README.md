# Atlas UI Prototype

This package is the first frontend prototype for the local Atlas web app.

It uses React, Ant Design, TanStack Query, and Rust-generated Atlas DTOs for the local search-to-detail workflow.

## Component Library

Ant Design is the selected component library for the prototype. See [docs/component-library-evaluation.md](docs/component-library-evaluation.md) for the decision note and retained comparison context.

## Development

Start the Rust local service in another terminal:

```bash
cargo run -p atlas-cli -- web --path-mode repo
```

Then run the Vite frontend:

```bash
npm install
npm run dev
```

Vite proxies `/api/*` to `http://127.0.0.1:4727` by default. Override that target with `ATLAS_API_PROXY` if the backend auto-selects another port:

```bash
ATLAS_API_PROXY=http://127.0.0.1:4728 npm run dev
```

## DTOs

The frontend imports a local aggregation file at `src/generated/atlas.ts`, which re-exports the checked-in Rust-generated bindings from `crates/atlas-app-model/bindings`.

Regenerate backend DTO bindings from the repository root with:

```bash
cargo test -p atlas-app-model export_typescript_bindings -- --ignored
```

Normal backend validation catches stale generated DTOs with:

```bash
cargo test -p atlas-app-model
```

Frontend validation commands:

```bash
npm run format:check
npm run lint
npm run typecheck
npm run test
npm run build
npm run verify
```
