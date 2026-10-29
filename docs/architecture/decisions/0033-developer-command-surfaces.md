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
- `dev-tools/source-contracts`: private TypeScript compiler research exposed
  directly through npm commands.

Rust developer commands do not launch Node and do not depend on the product CLI
or web bundle. Tools can exchange explicit JSON artifacts for future comparisons.
Both developer surfaces are validated in CI and excluded from releases.
atlas-dev and atlas-cli-support are private Cargo packages with dist disabled.

Development baseline and managed preview lifecycle belong to `atlas-dev baseline`.
The managed foreground owner retains the selected app's live child handle and
accepts serialized checkout-local refresh/stop requests. Receipts report state;
they never authorize terminating a recorded PID. Replacements use private frozen
executable copies, cheap selected-app compatibility checks and served readiness/UI
checksum checks. Source/model assets and the task database remain unchanged;
normal runtime/setup still owns schema updates.

Changed preview candidates pass a private cold-start probe before the old child
is stopped; unchanged candidates check the existing child. Serving admission
checks the index contract and actual web readiness, including ledger and every
stored profile, rather than repeating setup intent/source-freshness checks.

Candidate compilation is separate Node/shell orchestration under
`scripts/development`, exposed through just recipes. It can build the frontend and
product app, reuse a checksum-verified current bundle, and write source/build
provenance. The Rust baseline commands never dispatch that build script. Source
identity, executable/UI checksums and browser wire compatibility remain distinct.

Readiness probes operation-specific prerequisites without installing or migrating.
The retained developer tool exposes its compiled capabilities/schema and has a
source/content receipt. Explicit shared refresh freezes a validated app/tool pair,
and source checkout, selects release by default with an explicit dev override,
upgrades a private controlled seed, then delegates snapshot validation and atomic
publication to Rust. Existing task data and the installed personal workspace are
outside that maintenance operation. Long checks use the frozen checkout so caller
source edits may resume after capture; retained evidence identifies the candidate.

Validation entry points retain the shared shell path-selection policy, then run
one Node session under `scripts/validation`. That session owns a phase list with
one optional fixed frontend/Rust parallel pair,
input consistency and durable evidence; its process module owns live command
children and descendant cleanup. Shell retains only command launching and the
Windows cancellation handoff required when native termination bypasses Node signal
handlers. Browser candidate isolation retains its separate artifact/commit policy.
Plan/status inspection shares the execution definitions and treats legacy evidence
as unverified; it never automatically skips checks. Tooling tests have one exported
list and an independent direct CLI bootstrap, so testing the runner does not rely
on that runner successfully scheduling its own tests. The normal final-browser
entry point composes UI verification, packaging and the existing isolated runner
without duplicating candidate policy.

Maintained private TypeScript packages live under `dev-tools/`, with package
configuration at the package root, implementation under `src/`, and tests under
`tests/`. `src/cli/` owns command arguments, output and exit behavior; domain
modules own the reusable implementation. Source-contract fixtures and saved
large declaration graphs live in ignored caches, separate from implementation and small fixtures. `dev-tools/release`
owns notice, manifest, checksum and archive tooling. Operational shell and
PowerShell workflows and installers remain under `scripts/`. Rust developer
commands continue to live in the owning Cargo crates.

atlas-cli-support owns the presentation primitives used by both Rust CLIs:
path/progress argument vocabulary, JSON envelopes, and progress rendering.
The binary crates own command-specific grammar, routing, report presentation,
and exit mapping. Each binary maps shared path arguments to runtime path modes;
atlas-cli-support has no runtime dependency. Runtime path policy remains in
atlas-runtime, source analysis and audits remain in atlas-ingest, and artifact
inspection remains in atlas-index.
The support crate does not become a command dispatcher or business-logic facade.

## Consequences

Developer diagnostics build and run without preparing frontend assets or Node.
Product help and completions expose operational product commands. Contributor
documentation covers the developer commands; the product README covers setup
and user workflows. Moved commands have no compatibility aliases.

This extends ADR 0026's CLI presentation ownership to the two binary crates and
their shared presentation library; durable behavior still lives below the CLIs.
