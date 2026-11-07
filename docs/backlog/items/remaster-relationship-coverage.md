# Remaster relationship coverage

Status: in progress

## Problem

Atlas ingests Foundry remaster declarations, but name aliases and record-to-record
counterparts have different coverage. The current extractor accepts an old name
plus a resolved new-record reference as alias evidence. A remaster pair additionally
requires resolved root references on both sides and compatible source families.

During frontend review on 2026-11-05, the pinned PF2e source
`28d9d156f252d084371d994dc359509b49eadf18` produced 495 verified aliases and zero
remaster pairs in the full-corpus development workspace. The Remaster Changes
journal contains 560 data rows; 504 pass the current extractor's initial status/shape
gate and use plain old names. Attack of Opportunity
was retained as an alias for the general Reactive Strike action, without pairing
the legacy creature-glossary record with a remaster counterpart.

That baseline pair infrastructure therefore had fixture coverage while the inspected corpus
provided no populated counterpart data for version navigation or pair-based search
preference. This is separate from displaying explicit publication era metadata.

The [research and adversarial model review](../../research/remaster-information.md)
now records the source funnel, missing declaration kinds, schema/provenance limits
and consumer policy risks. The accepted direction retains useful journal-derived
lookup/context and a small explicit counterpart mapping, without result suppression.

## Current implementation and coverage audit

The 2026-11-06 [available-counterpart audit](../../research/remaster-counterpart-coverage.md)
found 612 declarations, 499 aliases and one approved two-record preference in the
current implementation. This is mapping coverage, not the number of available
legacy/remaster bodies. The complete 57-group / 118-record same-name census has
14 correspondence review candidates and 43 held cases. A broader old-term scan
found 19 journal-backed candidates: 15 individual renames, two mergers and two
class-scoped changes. Nine of the same-name candidates are core-publication
comparisons worth prioritizing separately. None was promoted during the audit.

Accepted simplification, 2026-11-06: maintain a small explicit Atlas counterpart
mapping for the 15 individual journal renames and nine core-publication cases,
plus the existing glossary pair. Validate exact keys and scope through normal
preparation. Remove automatic suppression, preference projections and all API, CLI
and browser controls. Search and discovery retain both independently matching
records. Journal extraction continues to supply useful aliases and compact context;
uncertain class, merger and adventure cases are not promoted by name matching.

## Accepted compact implementation

PR219 retains the existing alias and pair projections and adds one per-record
context table. It removes the experimental declaration/citation/binding/support/
term/capture framework, term inspection, missing-save modal and search annotations.
Normal record reads and the existing record-only graph command supply compact
relationships and useful journal navigation. Shared reader typography and inline
info controls remain. [ADR0067](../../architecture/decisions/0067-remaster-relationships.md)
owns the current design; the counts above describe the earlier research candidate.

Future coverage work can add reviewed exact mappings without building a new
relationship catalog. Uncertain class, merger and adventure identities remain
unmapped. Broader legacy-content imports, general UUID redirect handling and
saved-entry conversion are separate potential work. Saved records follow current
content at their exact chosen key.
