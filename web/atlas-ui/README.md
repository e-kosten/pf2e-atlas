# Atlas UI Prototype

This package is the first frontend prototype for the local Atlas web app.

It intentionally keeps one shared API/client/state layer and two visual implementations:

- Ant Design
- Mantine

Both implementations render the same search-to-detail workflow so the component-library comparison is grounded in the product surface rather than isolated widgets.

## Component-library comparison

The comparison intentionally shares Atlas behavior and isolates library-owned filters/results. See [docs/component-library-evaluation.md](docs/component-library-evaluation.md) for the evaluation matrix and guidance on when native library support should count as an advantage versus when custom Atlas composition is expected.

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
npm run test
npm run lint
npm run build
```
