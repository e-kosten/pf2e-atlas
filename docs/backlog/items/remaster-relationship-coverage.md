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

The pair infrastructure therefore has fixture coverage while the inspected corpus
provides no populated counterpart data for version navigation or pair-based search
preference. This is separate from displaying explicit publication era metadata.

The [research and adversarial model review](../../research/remaster-information.md)
now records the source funnel, missing declaration kinds, schema/provenance limits
and consumer policy risks. The accepted direction is checked typed declarations
with separate lookup aliases and narrowly approved preference relationships.

## Desired outcome

Deliver this as a focused PR, separate from the merged frontend presentation changes.
Preserve authored change statements, context, grouped endpoints and repeatable
source evidence, including missing legacy text and non-record terminology. Bind
records only through explicit evidence or an agreed reviewed-correspondence policy.
Keep publication metadata, declaration meaning, endpoint resolution and search
preference authority separate.

- Distinguish name renames, merges, replacements and verified document counterparts.
- Audit declarations whose legacy record is absent, ambiguous or in a different
  pack/context. A matching name alone must not silently establish equivalence
  between a class feature, general action and creature ability.
- Preserve source attribution and exact destinations. Keep useful aliases when
  the evidence cannot identify a legacy record.
- Add an ordinary-search choice: prefer a verified remaster result by default,
  or show all versions. Preserve independently matching saved members and exact
  encounter/owned identities under either choice.
- Present relation-specific context above rules, with source navigation and CLI
  parity; do not label every merger or replacement group an interchangeable version.
- Cover real pinned-source examples, including Attack of Opportunity, and report
  corpus-wide accepted, unresolved and rejected relationship coverage.
- Validate downstream counterpart presentation, search preference and ordinary
  lookup. Preserve saved state and compatible embeddings through any required
  artifact upgrade.

The baseline extractor and pair tables above are being replaced by the declaration,
binding and support ownership in [ADR0067](../../architecture/decisions/0067-remaster-relationships.md).
See [artifact evidence policy](../../architecture/artifact-contract.md) and
[runtime architecture](../../architecture/runtime.md). Keep navigation, name lookup
and preferred replacement authority distinct.

Additional reference providers, general UUID redirect handling and explicit
saved-entry conversion remain separate potential work. Saved records follow the
current content at their selected key; historical record snapshots and pinning
are excluded. Updating content at the same key does not authorize rebinding a
save to another key through a replacement relationship.
