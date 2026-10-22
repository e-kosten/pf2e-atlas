# Trait description content rights and attribution

Status: proposed
Priority: soon
Owner: unassigned
Last reviewed: 2026-10-20

## Problem

The [pinned provenance review](../../research/trait-description-provenance.md)
identifies the source bytes and upstream OGL/ORC notices but cannot assign a
publication or content license to each localized description. Some paragraphs
combine mechanics with setting material. Atlas's code license, the upstream
partnership statement and the presence of Community Use boilerplate do not
resolve those paragraph-level permissions.

## Outcome

Establish an evidence-backed permission/attribution policy for selected trait
descriptions, covering preparation, local display, exports and any contemplated
corpus distribution. Carry applicable notices and credits through the relevant
surface. Identify any permission that must be obtained separately.

Unknown and mixed material need an explicit decision: retain under an established
grant, replace with permitted content or restrict use. Do not infer a license
from names, localization keys, book dates, prose heuristics or the system pin.

## Acceptance

- Record publication and grant evidence for the paragraphs selected for use,
  including any translation/fallback attribution implications.
- Preserve the relevant OGL identification/copyright/license obligations and
  ORC attribution/reserved-material boundaries without relicensing either corpus.
- Resolve mixed setting/rules content and unknown sources with a reviewed policy.
- Implement any resulting ingest, storage, presentation or release changes in
  this separate workstream, with migration/state-preservation tests if needed.
- Do not bundle reference databases or source/model caches into releases while
  evaluating a future distribution policy.

The trait identity/reading implementation is delivered in PR93. This item tracks
content permissions and attribution, not new glossary pages or trait mechanics.
