# Contextual filter option counts

Status: done
Priority: soon
Owner: unassigned
Last reviewed: 2026-10-19

## Completed outcome

PR60 delivered shared contextual option counts, self-exclusion, stable picker updates and supported-context behavior. Query-text counts remain a separate deferred task.

The scope below records the original task; it is not outstanding implementation.

## Original scope

Restore contextual match counts alongside categorical options in common and
advanced pickers. PR55 completed the reference browser using count-free vocabulary;
the user explicitly requested counts as a subsequent task.

Reuse index discovery/eligibility contracts. Define which parent records each
number counts, self-exclude the edited condition correctly and preserve same-child
constraints. Field-state statistics are not option-match counts. For unsupported
OR/NOT contexts, retain useful vocabulary without counts or warning prose;
unavailable counts must not appear as zero or disable valid filters.

Preserve searchable/paged/selected values and mixed include/exclude editing.
Validate conjunctions, exclusions, complex groups, embedded scopes and shared
saved-list/encounter pickers. Check incomplete drafts, loading/cancellation and
full-corpus request cost. Broader complex-context support requires an explicit
backend semantics decision; do not guess counts in the frontend.

This concerns filter context. Query-text-aware counts remain a separate
[deferred task](../../items/rust-web-text-scoped-filter-counts.md).
Historical design: PR55 plan follow-up (historical local plan: `scratch/plans/archive/2026-10/2026-10-12-reference-workspace-and-app-chrome.md#follow-up-after-pr55-restore-filter-option-counts`).
