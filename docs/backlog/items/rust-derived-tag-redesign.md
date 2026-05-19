# Rust Derived-Tag Runtime And Editorial Redesign

Status: planned
Priority: later
Owner: unassigned
Last reviewed: 2026-05-19

## Problem

Derived tags remain useful as a product concept, but the Rust artifact model changes the surrounding vocabulary: record families replace broad category/subcategory assumptions, explicit source axes are available, and search/filter discovery has stronger typed foundations.

The next implementation should redesign the retained tag model for the Rust runtime instead of porting the previous editorial implementation directly.

## Desired Outcome

Define and implement a Rust-owned derived-tag runtime and editorial workflow after the core CLI/search path is stable.

The redesign should decide:

- which tag concepts remain runtime filter axes
- which concepts become authored taxonomy or editorial-only metadata
- which concepts retire because record family, traits, metadata, metrics, or graph context cover the use case better
- how authored assignments, exemplars, reviews, and generated candidates should be represented on disk
- which CLI and future TUI workflows are needed for review, assignment, evaluation, and discovery
- how agent workflows should support tag audit, batch assignment, exemplar review, seed migration, and rule maintenance
- how retained tag concepts appear in search, filter discovery, and record presentation

## Constraints

- Add an `atlas-tags` crate only when the accepted runtime/editorial model starts implementation.
- Do not depend on retired implementation files as the runtime model.
- Keep high-churn candidate discovery and clustering behind stable runtime tag consumption.
- Preserve useful authored knowledge only through a deliberate migration or seed format.
- Preserve useful workflow intent from retired tag-maintenance skills, but rebuild it around Rust CLI commands and Rust-owned storage.

## Notes

This item preserves the useful intent from retired derived-tag backlog items around concept modeling, index/service boundaries, assignments layout, manifest metadata, review questions, tagging command organization, category relevance, and editorial workflows.

## Related

- [Runtime architecture](../../architecture/runtime.md)
- [Artifact contract](../../architecture/artifact-contract.md)
- [ADR 0026: Rust CLI product surface](../../architecture/decisions/0026-rust-cli-product-surface.md)
