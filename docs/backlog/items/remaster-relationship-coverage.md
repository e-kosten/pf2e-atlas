# Remaster relationship coverage

Status: proposed

## Problem

Atlas ingests Foundry remaster declarations, but name aliases and record-to-record
counterparts have different coverage. The current extractor accepts an old name
plus a resolved new-record reference as alias evidence. A remaster pair additionally
requires resolved root references on both sides and compatible source families.

During frontend review on 2026-11-05, the pinned PF2e source
`28d9d156f252d084371d994dc359509b49eadf18` produced 495 verified aliases and zero
remaster pairs in the full-corpus development workspace. The Remaster Changes
journal's 504 candidate table rows used plain old names. Attack of Opportunity
was retained as an alias for the general Reactive Strike action, without pairing
the legacy creature-glossary record with a remaster counterpart.

The pair infrastructure therefore has fixture coverage while the inspected corpus
provides no populated counterpart data for version navigation or pair-based search
preference. This is separate from displaying explicit publication era metadata.

## Desired outcome

Investigate this as a focused PR, separate from the frontend presentation changes.
Establish a source-backed policy for connecting authored rename/replacement
information to specific available records, with explicit ambiguity and absence.

- Distinguish name renames, merges, replacements and verified document counterparts.
- Audit declarations whose legacy record is absent, ambiguous or in a different
  pack/context. A matching name alone must not silently establish equivalence
  between a class feature, general action and creature ability.
- Preserve source attribution and exact destinations. Keep useful aliases when
  the evidence cannot identify a legacy record.
- Cover real pinned-source examples, including Attack of Opportunity, and report
  corpus-wide accepted, unresolved and rejected relationship coverage.
- Validate downstream counterpart presentation, search preference and ordinary
  lookup. Preserve saved state and compatible embeddings through any required
  artifact upgrade.

Relevant owners are `atlas-ingest/src/source_aliases.rs`, the index's
`verified_aliases` and `remaster_pairs`, and the shared app-service remaster
projection. See [artifact evidence policy](../../architecture/artifact-contract.md)
and [runtime architecture](../../architecture/runtime.md). Do not weaken the
existing pair contract solely to populate the UI; establish and test the intended
identity/evidence contract first.
