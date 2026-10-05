# Rust Foundry JSON Field Audit

Status: done
Completed: 2026-10-05

The offline source audit is implemented as schema discovery, snapshots, and diffs through `atlas source schema`. It inventories all selected manifest-declared source documents and nested input paths, JSON types, counts, duplicate object members, and bounded examples. A pinned observed-corpus baseline is checked in and compared in CI.

The earlier proposal to require field-to-model ownership and explicit omission ledgers is retired. [ADR 0037](../../../architecture/decisions/0037-source-schema-discovery.md) replaces the exhaustive source-leaf receipt framework with discovery diagnostics. Product-driven modeling retains focused parser, projection, and SQLite persistence tests. Registration-only possibilities are outside the observed-corpus snapshot.

See [contributor workflow](../../../../CONTRIBUTING.md) and [the pinned schema baseline](../../../../contracts/source-schema/v1/README.md).
