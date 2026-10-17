# Developer tooling

Maintained private packages for contributor and release workflows:

- [source-contracts](./source-contracts/README.md): Foundry declaration discovery,
  Rust source generation, corpus sampling and parser comparisons.
- [release](./release/README.md): dependency notices, manifests, checksums and
  archive validation.

Each package owns its dependencies, TypeScript configuration and npm commands.
Implementation lives in `src/`, command entry points in `src/cli/`, and tests in
`tests/`. Fixtures and saved snapshots have separate directories when needed.
Generated JavaScript lives in ignored package-local `dist/` directories. These
packages and their Node dependencies are excluded from Atlas distributions.

Operational shell and PowerShell workflows and installers live in `scripts/`.
Rust diagnostics live in `crates/atlas-dev` and use the owning Rust libraries.
