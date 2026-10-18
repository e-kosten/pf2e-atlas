# Database design and release readiness

Status: planned
Priority: soon
Owner: unassigned

## Outcome

Review the database from first principles before 1.0, adversarially testing the
current schema and storage boundaries against future product requirements.
Breaking changes are permitted now: no existing runtime state needs preserving.
Review the proposed end state with the user before implementation.

## Scope

- Inspect actual schema, keys, constraints, JSON, duplication, relationships and
  query patterns. Challenge the two-database split against single and attached
  database alternatives, including operational complexity and data lifetimes.
- Separate durable user state, source-derived records and rebuildable search
  data. Define source identity and missing/changed-reference behavior.
- Test future parties, campaigns, users, memberships and GM/player roles against
  ownership boundaries without prematurely implementing authentication.
- Design tested migrations and upgrade guarantees, atomic publication,
  interrupted-upgrade recovery, backups, concurrent access and background builds.
  Prefer additive expansion; support necessary transformations without data loss.
- Include [embedding reuse](embedding-reuse-preparation-upgrades.md): presentation
  changes must not force inference when exact model inputs remain unchanged.
- Evaluate storage growth, model replacement and the ability to keep the
  application usable while refreshing derived data.

## Deliverable

An evidence-backed review, proposed end-state schema and migration/rebuild
boundaries, tradeoffs and representative upgrade scenarios. Agree release
compatibility promises and validation before implementing the redesign.
