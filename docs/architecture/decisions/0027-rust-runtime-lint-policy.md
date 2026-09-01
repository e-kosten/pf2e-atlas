# 0027 Rust Runtime Lint Policy

## Status

Accepted.

## Context

The initial Rust migration used standard Clippy warnings-as-errors plus crate-level `unsafe_code` denies. That was appropriate while runtime APIs were still taking shape, but PF2e Atlas now has durable Rust ownership for ingest, indexing, runtime path resolution, search orchestration, and CLI output.

As those boundaries stabilize, production code should not rely on panic-oriented shortcuts for recoverable or externally influenced states. Panics remain useful in tests, where `unwrap`, `expect`, and explicit `panic!` often make assertions easier to read.

## Decision

Lint production and non-production targets as complementary, non-overlapping
sets. Runtime libraries and binaries carry the strict policy:

```bash
cargo clippy --workspace --lib --bins -- -D warnings \
  -D clippy::dbg_macro \
  -D clippy::unwrap_used \
  -D clippy::expect_used \
  -D clippy::panic \
  -D clippy::unimplemented \
  -D clippy::todo \
  -D clippy::unreachable
```

Tests, benches, and examples retain warnings-as-errors and the `dbg!` ban
without the runtime panic denies:

```bash
cargo clippy --workspace --tests --benches --examples -- \
  -D warnings -D clippy::dbg_macro
```

Do not combine an all-targets pass with the runtime pass: that recompiles and
relints production targets while obscuring which policy applies. A small
fixture must prove that panic-oriented production code fails and equivalent
assertion-oriented test code remains permitted.

Both target sets reject `dbg!`, which is useful during local debugging but emits file/line diagnostics to stderr and should not be committed in runtime code or tests.

The runtime-only gate applies to library and binary targets, not test targets. Runtime code must propagate errors, return explicit fallback values, or encode impossible states in types rather than panicking. Tests may continue to use assertion-oriented `unwrap`, `expect`, and `panic!` when that keeps the test clearer.

The policy is enforced by git hooks and release checks, and should be treated as part of the repository's normal Rust verification gate.

## Consequences

Runtime code must model bad states explicitly, including internal invariant failures from embedding/vector generation, artifact serialization, path resolution, progress rendering, parser stack handling, and graph/index writes.

Future production panic exceptions should be rare and deliberate. If a panic-like behavior is truly required, prefer a domain-specific error, validation diagnostic, or typed impossible-state representation before considering a lint exception.
