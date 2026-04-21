# Backlog

This folder is the tracked home for durable future work. It exists to separate stable backlog context from disposable working plans under `scratch/plans/`.

`scratch/plans/future-plans.md` has not been migrated yet. Leave it untouched until its contents have been reviewed and moved intentionally.

## Structure

- `README.md`
  Canonical backlog index, status vocabulary, and triage guidance.
- `items/`
  One file per substantial backlog item that needs durable context.
- `history/`
  Optional snapshots or archived status rolls when historical tracking is useful.

## Status Vocabulary

Use one of these statuses for backlog entries:

- `proposed`
- `planned`
- `in_progress`
- `blocked`
- `deferred`
- `done`
- `superseded`

## Buckets

Use these buckets in the live index:

- `Now`
- `Soon`
- `Later`
- `Done / Superseded`

Keep inline entries short. If an item needs more than a few lines of explanation, create an item file under `items/` and link to it here.

## Item Shape

Inline items in this file should usually have:

- short title
- 1-3 sentence summary of the problem or desired outcome
- current status
- optional link to a deeper item file

Example:

```md
## Soon

- [Ontology browser naming](./items/ontology-browser-naming.md)
  Replace internal-facing field names with user-facing labels in explorer surfaces.
  Status: proposed.
```

## Rules Of Thumb

- `scratch/` may be lost; `docs/backlog/` should not.
- Do not store durable architecture rules here. Move those to `docs/architecture/` or `docs/architecture/decisions/`.
- Do not create one file per tiny note. Keep small items inline in `README.md`.
- Create `items/<slug>.md` only when the item has meaningful constraints, design context, or cross-cutting scope.
- Temporary implementation plans may link from backlog items while work is active, but durable backlog entries should not depend on `scratch/` files remaining available.

## Current Index

## Now

- No tracked backlog items yet.

## Soon

- No tracked backlog items yet.

## Later

- No tracked backlog items yet.

## Done / Superseded

- No tracked backlog history yet.
