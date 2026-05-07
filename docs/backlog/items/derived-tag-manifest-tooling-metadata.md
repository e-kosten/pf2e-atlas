# Derived Tag Manifest Tooling Metadata

Status: proposed  
Priority: later  
Owner: unassigned  
Last reviewed: 2026-04-21

## Problem

A retired branch, `feat/tag-registry-manifest`, included one remaining idea that did not clearly belong in the current architecture but was interesting enough to preserve for later evaluation:

- the shared derived-tag category manifest might optionally carry repo-layout and owner-file metadata for each managed category

Current `main` already has the important architectural consolidation in [src/tags/manifest.ts](../../src/tags/manifest.ts):

- managed categories
- editorial ordering
- registration ordering
- capability flags such as `supportsLegacySeedMigration`

The branch went further by attaching metadata such as:

- per-category assignment file path
- per-category exemplar file path
- per-category authored-rule file path
- per-category legacy-rule file path
- per-category ontology file path
- review-registry file paths

That extra metadata was not necessary to complete the category-manifest refactor, and it creates tighter coupling between the manifest and the current repo layout. But it may still be useful for future tooling, editorial navigation, or manifest-driven maintenance tasks.

## Desired Outcome

Decide later whether the shared derived-tag manifest should remain a narrow category/capability registry or grow a small amount of optional tooling-facing metadata about current owner files.

If this idea is adopted, the goal should be:

- make category-scoped tooling easier to write
- reduce repeated hardcoded category-to-file lookup tables in scripts or editorial helpers
- keep the metadata clearly positioned as tooling/support metadata, not as semantic model ownership

## Constraints

- Do not treat this as a required architecture gap. Current `main` already has the important part of the category-manifest refactor.
- Do not add repo-path metadata to the manifest unless there is a concrete user such as tooling, editorial navigation, validation, or manifest generation.
- Treat repo-layout metadata as more volatile than category identity and ordering.
- Reevaluate this idea in the context of the planned future-state concept/projection model before adopting it broadly.

## Notes

### Why the idea is interesting

One central manifest that can answer both:

- "which managed categories exist?"
- "where do this category's current authored assets live?"

could be useful for:

- category-scoped tooling
- editorial navigation helpers
- generated docs or maintenance checks
- reducing duplicated file-path lookup tables in scripts

### Why it may not be worth doing

The manifest today is a durable architectural owner for category identity, ordering, and capability flags.

Adding repo-layout metadata would make it more opinionated about the current implementation structure:

- file paths churn more often than category identity
- the metadata would describe current source layout, not domain meaning
- future refactors could invalidate the added value quickly

So this idea should only land if it has a real consumer, not just because the branch happened to carry it.

### Relationship to the future-state plan

This idea is explicitly subordinate to the longer-term derived-tag future-state planning captured in:

- [Derived Tag Canonical Concept Model](../../../scratch/derived-tag-ontology/derived-tag-concept-model.md)
- [Derived Tag Concept Mapping Summary](../../../scratch/derived-tag-ontology/derived-tag-concept-mapping-summary.md)

Those planning artifacts may change which registry metadata is actually useful and where it should live.

The most likely steady-state split is still:

- semantic ownership in canonical concepts and category projections
- optional tooling/navigation metadata only where current authored file ownership actually needs to be discoverable

### Source context

This note preserves the one interesting leftover idea from the retired clean branch `feat/tag-registry-manifest` after the main architectural value of that branch was found to be largely superseded by the current `src/tags/manifest.ts` structure.

## Related

- [Derived-tag assignments layout](./derived-tag-assignments-layout.md)
- [Tagging tooling reorganization](./tagging-tooling-reorg.md)
- [Derived Tag Canonical Concept Model](../../../scratch/derived-tag-ontology/derived-tag-concept-model.md)
- [Derived Tag Concept Mapping Summary](../../../scratch/derived-tag-ontology/derived-tag-concept-mapping-summary.md)
