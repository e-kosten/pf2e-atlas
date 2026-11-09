# Developer tooling

Maintained private packages for contributor and release workflows:

- [source-contracts](./source-contracts/README.md): Foundry declaration discovery,
  Rust source generation, corpus sampling and parser comparisons.
- [OpenAPI](../web/atlas-ui/README.md): reproducible export of the Atlas HTTP
  contract and generation of the browser client and models.
- [release](./release/README.md): dependency notices, manifests, checksums and
  archive validation.

Each package owns its dependencies and npm commands. The source-contracts and
release packages keep implementation in `src/`, entry points in `src/cli/`,
tests in `tests/`, and compiled JavaScript in ignored package-local `dist/`
directories. OpenAPI uses a small JavaScript driver at its package root.
The [dice prototype](dice-prototype/README.md) retains exploratory comparisons.
These packages and their Node dependencies are excluded from Atlas distributions.

Operational shell and PowerShell workflows and installers live in `scripts/`.
Thin shell launchers in `scripts/development` invoke the retained Rust `atlas-dev`
baseline commands to publish builds/data and initialize private workspaces; see
[development baselines](../docs/development/setup-and-preview.md#reusable-development-baseline).
Rust diagnostics live in `crates/atlas-dev` and use the owning Rust libraries.
