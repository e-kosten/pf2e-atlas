# ADR 0033: Developer Command Surfaces

Status: accepted
Date: 2026-10-06

## Context

Source analysis, raw path audits, and artifact inspection support implementation
and research. They share Rust libraries with the product but do not serve the
ordinary search or web setup workflow. Foundry declaration extraction requires
the TypeScript compiler and its own private dependency environment.

## Decision

The three command owners are:

- `atlas`: user and agent workflows, setup, index build/check/validate, retrieval,
  web startup, and other operational product commands.
- `atlas-dev`: locally built Rust diagnostics using atlas-ingest, atlas-index,
  and atlas-runtime. Its initial tree is source analyze, source audit-paths, and
  index inspect.
- `scripts/source-contracts`: private TypeScript compiler research exposed
  directly through npm commands.

Rust developer commands do not launch Node and do not depend on the product CLI
or web bundle. Tools can exchange explicit JSON artifacts for future comparisons.
Both developer surfaces are validated in CI and excluded from releases.
atlas-dev and atlas-cli-support are private Cargo packages with dist disabled.

atlas-cli-support owns the presentation primitives used by both Rust CLIs:
path/progress argument vocabulary, JSON envelopes, and progress rendering.
The binary crates own command-specific grammar, routing, report presentation,
and exit mapping. Runtime path policy remains in atlas-runtime, source analysis
and audits remain in atlas-ingest, and artifact inspection remains in atlas-index.
The support crate does not become a command dispatcher or business-logic facade.

## Consequences

Developer diagnostics build and run without preparing frontend assets or Node.
Product help and completions expose operational product commands. Contributor
documentation covers the developer commands; the product README covers setup
and user workflows. Moved commands have no compatibility aliases.

This extends ADR 0026's CLI presentation ownership to the two binary crates and
their shared presentation library; durable behavior still lives below the CLIs.
