# Architecture Cleanup Rebaseline Checklist

## Tags Ownership And Facades

- [x] Remove the dependency from `src/tags/editorial/**` onto `src/app/ontology/entity-record.ts`.
- [x] Move the shared entity-record contract/helpers to an owner outside `src/app/`.
- [x] Migrate non-tag callers away from `src/tags/editorial/*` internals to approved facade exports.
- [x] Keep the approved non-tag facade surface minimal and explicit.

## Tags Internal Imports And Compatibility Shims

- [x] Migrate first-party internal imports off top-level `src/tags/editorial/*.ts` shims where split owners exist.
- [x] Migrate first-party internal imports off top-level `src/tags/runtime/*.ts` shims where split owners exist.
- [x] Migrate CLI/tests off `src/tags/discovery/discovery-reviewed-records.ts` onto `src/tags/reviews/discovery-reviewed-records.ts`.
- [x] Remove compatibility shims that no longer have a justified compatibility role.

## Lint, Docs, And Enforcement

- [x] Align non-tag tag-internal lint restrictions with the documented boundary story.
- [x] Fix stale JSON decode allowlist entries to point at current owner paths.
- [x] Remove dead migration-era allowlist entries.
- [x] Update architecture docs only where code and intended boundaries materially changed.
- [x] Update architecture lint tests for the tightened routes.

## TUI Search Refactor

- [x] Materially decompose `src/tui/search-screen/workspace-actions.ts` into smaller owner modules.
- [x] Preserve existing user-visible search/editor behavior.
- [x] Reduce obvious duplicated TUI helper ownership, including repeated formatting helpers where practical.
- [x] Keep the `src/tui/search/` vs `src/tui/search-screen/` layering coherent after the split.

## Validation

- [x] `npm run build`
- [x] `npm test`
- [x] Re-read this checklist and confirm each item against the final repo state before landing.

## Notes

- Remaining top-level `src/tags/runtime/*.ts`, `src/tags/editorial/*.ts`, and `src/tags/discovery/discovery-reviewed-records.ts` compatibility files were intentionally preserved where they still serve documented compatibility or public-surface roles; no first-party internal callers remain on those shims.
- No architecture doc edits were required after the final re-check because the code and enforcement changes were brought back into alignment with the existing documented boundary story.
