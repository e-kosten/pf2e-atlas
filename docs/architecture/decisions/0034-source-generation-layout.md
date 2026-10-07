# ADR 0034: Source Generation Layout

Status: accepted
Date: 2026-10-06

## Context

The bounded equipment trial generates four physical fields and their equipment
refinement. As generation expands, both saved declaration evidence and Rust
output need cohesive modules with shared owners. Generating each family
independently would duplicate shared declarations and Rust types.

## Decision

Private `scripts/source-contracts/snapshots` owns saved generation inputs. One
manifest records source identity and ordered module files. Each module snapshot
contains its roots, deferred field names and canonical graph nodes. Nodes occur
once across snapshots; references retain their global declaration identities.
The selection orders shared base roots before refinements so base modules own
their shared closure and refinement modules add only their additional nodes.
Roots for each module are contiguous; interleaving would change ownership on
manifest reload and is rejected.

Load the whole selected graph before emitting Rust. A global owner table shares
equivalent value structures under the explicit pre-default presence policy.
Input optional/null/undefined facts and declaration provenance remain intact.
Output partitioning follows module ownership with explicit cross-module imports;
it is not separate per-family generation. Content modules are items/common,
items/flags, items/traits, physical, items/equipment and rules/predicate.
Generate only modules with definitions or existing children.

`atlas-ingest/src/source_model/generated` owns generated Rust and its explicit
module/re-export indexes. Handwritten source presence, ordered values, parsing
primitives and slice composition stay outside this directory. Crate entrypoints
expose the intentional API; Rust execution has no Node dependency.

The private npm `generate` command replaces the single-file experimental command.
Saved-manifest mode checks the canonical input set and checks/regenerates Rust;
graph mode also checks/regenerates
the partitioned snapshots using the current bounded selection recipe. Freshness
checks cover the full expected file sets, including missing/obsolete files and
metadata changes that value-type sharing can hide. Regeneration removes obsolete
generator-owned files and refuses unmanaged files or symlinks in artifact
directories. Format and preflight all outputs before writing.

## Consequences

New families extend the module layout without copying existing shared closures.
Generated directories are exclusively tool-owned; contributor notes belong
outside them. Source updates intentionally regenerate inputs and Rust together.
This establishes layout and editing rules, not full-schema generation support,
full Foundry admission or pipeline/storage/UI adoption. Shared Item fields and
four physical/equipment fields are partial source slices; additional constructs
and coverage need follow-up work.
