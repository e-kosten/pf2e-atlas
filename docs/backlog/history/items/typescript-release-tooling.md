# TypeScript release tooling

Status: done
Last reviewed: 2026-10-06

The private `scripts/release` npm package owns release notices, JSON manifests,
checksums, archive inspection and shared smoke-test fixture construction.
Implementation and tests use strict TypeScript with emitted Node entry points.
Shell and PowerShell retain their installer and release-preparation workflows.

Maintained release scripts and workflows require no Python. Release jobs build
the package before use; ordinary PR CI checks Linux, macOS and Windows. Package
dependencies are developer tooling and do not enter product distributions.

Manifest formats, checksums and notice content are preserved. Archive exclusion
tests regenerate checksum metadata before validation so they reach the archive
inspection path. The separate source-contracts conversion and future Rust
`atlas-dev` command separation have independent ownership and review boundaries.
