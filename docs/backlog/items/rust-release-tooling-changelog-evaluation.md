# Rust Release Tooling And Changelog Evaluation

Status: proposed
Priority: later
Owner: unassigned
Last reviewed: 2026-09-08

## Problem

Atlas spans a Cargo workspace and a frontend application, but its prerelease workflow does not yet have a settled way to record known compatibility changes, aggregate release notes, choose application versions, and create release tags and packages. Knope may be useful broader Rust release tooling, but it has not been selected or evaluated against the existing workflow.

Application SemVer is separate from the rebuildable derived-index artifact contract, schema, and manifest versions, and from the migration sequence for irreplaceable user local-state data. Release tooling must not infer or collapse those independent compatibility decisions.

## Desired Outcome

Evaluate Knope alongside the repository's existing release and validation tooling, then recommend whether to adopt it or retain a simpler workflow. The evaluation should cover:

- Cargo workspace and frontend application versioning, changelog aggregation, tags, and release packaging
- prerelease and integration-branch behavior
- a lightweight per-PR Markdown release-note policy recording what changed, affected surface, whether it is breaking, and whether the required action is none, derived-index rebuild, user-state migration, or a manual step
- explicit `no release needed` cases for changes such as internal tests or documentation with no user-visible or compatibility effect
- human review of note presence and content without inferring version bumps from source paths
- later aggregation of accepted notes into application changelogs and release notes
- independent treatment of application SemVer, derived-index format compatibility, and local-state migration versions

## Constraints

- This is a deferred evaluation. Recording it during PR8 cleanup does not start
  release-tooling work, change current CI stabilization, redesign databases, or
  alter integration rollout acceptance.
- Tool choice remains undecided. Compare Knope with the existing workflow only when this item is scheduled; do not install tooling or start a release-automation project beforehand.
- Do not weaken runtime format rejection, artifact integrity checks, derived-index rebuild guidance, or local-state migration and preservation guarantees.
- Avoid a blanket note requirement for changes that are explicitly reviewed as having no release impact.

## Related

- [Artifact contract](../../architecture/artifact-contract.md)
- [Rust local-state import and export](./rust-local-state-import-export.md)
