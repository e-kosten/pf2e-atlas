# Rust Artifact Ownership Hardening

Status: done

## Context

The Rust artifact contract is functional, but architecture review identified ownership drift risks around schema descriptors, writer SQL, reader SQL, validation expectations, metric policy, default reference graph policy, and vector blob storage encoding.

## Outcome

- Rich artifact table and column descriptors generate SQLite DDL and drive insert SQL, select SQL, and schema-validation expectations.
- Ingest and index consume descriptor-owned artifact schema and storage helpers instead of maintaining independent schema inventories.
- Metric definitions are typed Rust policy used by ingest extraction, presentation, and ingest-side metric audits.
- Default reference graph behavior is defined once in Rust policy and lowered by index code, rather than stored as a database boolean.
- `f32` vector blob encoding and decoding live in the artifact storage boundary.

## Planning Artifact

The implementation plan was `scratch/plans/rust-artifact-ownership-hardening.md`.
