# ADR 0034: Source Generation Layout

Status: accepted
Date: 2026-10-06

## Context

The complete portfolio needs shared type owners and a reproducible relationship
to upstream declarations. Committing both a complete declaration graph and its
Rust translation adds substantial generated diff volume. Rust consumers need
only the generated Rust, while declaration graphs remain useful developer evidence.

## Decision

Private `dev-tools/source-contracts/source-pin.json` records the upstream repository,
immutable commit, source version/digest/file count, compiler version and expected
specific rule keys. The npm lockfile pins compiler and declaration dependencies.
Authored representation policies remain maintained TypeScript code.

The private `generate` command acquires the exact source into an isolated export
under ignored `.cache/source-contracts`. A supplied source is checked against the
pin and copied; its checkout and dependency installation are not modified. Cold
acquisition fetches the immutable commit into a cache-owned bare repository and
exports only the source-identity inputs. Source bytes are checked on every use.
The isolated export resolves declarations through this package's locked dependencies.

Every generation/check invocation extracts the complete schema afresh. Type
graphs, trait catalogs, generation inputs and extraction summaries stay in the ignored cache;
cached graphs are never generation authority. Source identity, complete extraction,
compiler version, rule keys and registered family sets are checked before writing
Rust. Large declaration snapshots and their loading command are removed.
Ordinary fixture tests are offline; the dedicated CI generation check fetches
the source when needed and proves the complete source-to-Rust path.

Load the whole selected graph before emitting Rust. Shared base roots precede
refinements, and a global owner table shares equivalent pre-default value shapes.
Declaration optional/null/undefined facts and provenance remain in extraction
evidence. Family source/system roots reserve their modules even when first reached
through another family's embedded nullable/optional source.

`atlas-ingest/src/source_model/generated` owns committed Rust and module/re-export
indexes. Content modules include shared Item/Actor/creature/physical components,
family sources/systems, other document kinds and specific rules. Aggregate Actor/Item
and rule dispatch have separate modules. Handwritten source presence, ordered values,
parsing primitives and composition stay outside the generated directory.
Rust builds and application execution have no Node dependency.

Freshness checks cover the full Rust file set, including missing/obsolete files.
Regeneration formats and preflights outputs before writing, removes only obsolete
generator-owned files, and refuses unmanaged files or symlinks in output directories.

Corpus comparison re-extracts the pinned source, checks maintained Rust freshness,
and gives the actual crate probe a temporary generation input for the independent
value/presence/additional-member oracle. No saved declaration snapshot is required.

## Consequences

Review source-pin, generator/policy changes and Rust diffs when updating upstream.
Declaration-only changes still alter the checked source digest even when Rust
value sharing hides them. To inspect schema differences, extract both revisions
and compare their ignored graphs. Cache misses require Git/network access;
warm cached exports or a supplied matching source support offline regeneration.
Git and tar are private contributor acquisition prerequisites.

The portfolio covers all 47 extracted roots, all 24 Item/eight Actor families and
42 specific rules. Existing field-level projections share owners with full sources.
This establishes source models, not Foundry admission or pipeline/storage/UI adoption.
Corpus rejections remain visible without repair, exclusion or a selected drop policy.
